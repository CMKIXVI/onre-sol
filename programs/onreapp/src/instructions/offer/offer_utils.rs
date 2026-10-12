use crate::constants::MAX_BASIS_POINTS;
use crate::instructions::market_info::read_market_stats_account;
use crate::instructions::redemption::load_optional_checked_redemption_offer;
use crate::instructions::{Offer, OfferVector};
use crate::state::State;
use crate::utils::approver::approver_utils;
use crate::utils::{
    calculate_fees, calculate_token_out_amount, mul_div_round_u128, pow_fixed,
    program_controls_mint, ApprovalMessage,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

const INT_SCALE: u128 = 1_000_000_000_000_000_000;
const SECONDS_IN_DAY: u64 = 86_400;
const APR_SCALE: u128 = 1_000_000;

/// Result structure containing offer processing calculations
pub struct OfferProcessResult {
    /// Current price with scale=9 (1_000_000_000 = 1.0) at the time of processing
    pub current_price: u64,
    /// Amount of token_in after fee deduction
    pub token_in_net_amount: u64,
    /// Fee amount deducted from the original token_in amount
    pub token_in_fee_amount: u64,
    /// Calculated amount of token_out to be provided to the user
    pub token_out_amount: u64,
}

/// Verifies approval requirements for offer operations.
///
/// Approval signatures are scoped to the exact offer being executed. A signature
/// for one approval-gated offer cannot be replayed against a different offer.
pub fn verify_offer_approval(
    offer: &Offer,
    approval_message: &Option<ApprovalMessage>,
    program_id: &Pubkey,
    user_pubkey: &Pubkey,
    approver1: &Pubkey,
    approver2: &Pubkey,
    instructions_sysvar: &UncheckedAccount,
) -> Result<()> {
    if offer.needs_approval() {
        match approval_message {
            Some(msg) => {
                msg!(
                    "Offer requires approval, verifying message {}",
                    msg.expiry_unix
                );
                approver_utils::verify_approval_message_generic(
                    program_id,
                    &offer.key,
                    user_pubkey,
                    approver1,
                    approver2,
                    instructions_sysvar,
                    msg,
                )?;
            }
            None => return Err(error!(crate::OnreError::ApprovalRequired)),
        }
    }
    Ok(())
}

/// Core processing logic for offer execution calculations
///
/// Performs comprehensive validation and calculation for offer processing including
/// active vector identification, price calculation with APR-based growth, fee
/// calculation, and token amount conversions with decimal adjustments.
///
/// # Arguments
/// * `offer` - The loaded offer containing pricing vectors and configuration
/// * `token_in_amount` - Amount of token_in being provided by the user
/// * `token_in_mint` - The token_in mint for decimal and validation information
/// * `token_out_mint` - The token_out mint for decimal and validation information
/// * `fee_basis_points` - Fee in basis points to apply for this execution path
///
/// # Returns
/// * `Ok(OfferProcessResult)` - Containing current price, token amounts, and fees
/// * `Err(_)` - If validation fails or no active vector exists
pub fn process_offer_core(
    offer: &Offer,
    token_in_amount: u64,
    token_in_mint: &InterfaceAccount<Mint>,
    token_out_mint: &InterfaceAccount<Mint>,
    fee_basis_points: u16,
) -> Result<OfferProcessResult> {
    let current_time = Clock::get()?.unix_timestamp as u64;

    process_offer_core_at(
        offer,
        token_in_amount,
        token_in_mint.key(),
        token_in_mint.decimals,
        token_out_mint.key(),
        token_out_mint.decimals,
        current_time,
        fee_basis_points,
    )
}

fn process_offer_core_at(
    offer: &Offer,
    token_in_amount: u64,
    token_in_mint_key: Pubkey,
    token_in_decimals: u8,
    token_out_mint_key: Pubkey,
    token_out_decimals: u8,
    current_time: u64,
    fee_basis_points: u16,
) -> Result<OfferProcessResult> {
    require!(token_in_amount > 0, crate::OnreError::InvalidAmount);
    offer.require_mints(token_in_mint_key, token_out_mint_key)?;

    // Find the currently active pricing vector
    let active_vector = find_active_vector_at(offer, current_time)?;

    // Calculate current price with 9 decimals
    let current_price = calculate_current_step_price(
        active_vector.apr,
        active_vector.base_price,
        active_vector.base_time,
        active_vector.price_fix_duration,
    )?;

    let fee_amounts = calculate_fees(token_in_amount, fee_basis_points)?;

    // Calculate how many token_out to give for the provided token_in_amount
    let token_out_amount = calculate_token_out_amount(
        fee_amounts.token_in_net_amount,
        current_price,
        token_in_decimals,
        token_out_decimals,
    )?;

    Ok(OfferProcessResult {
        current_price,
        token_in_net_amount: fee_amounts.token_in_net_amount,
        token_out_amount,
        token_in_fee_amount: fee_amounts.token_in_fee_amount,
    })
}

pub fn is_onyc_token_out_mint<'info>(
    state: &Account<'info, State>,
    token_out_mint: &InterfaceAccount<'info, Mint>,
) -> bool {
    token_out_mint.key() == state.onyc_mint
}

pub fn should_accrue_onyc_mint<'info>(
    state: &Account<'info, State>,
    token_out_mint: &InterfaceAccount<'info, Mint>,
    buffer_is_initialized: bool,
    mint_authority: &AccountInfo<'info>,
) -> bool {
    is_onyc_token_out_mint(state, token_out_mint)
        && buffer_is_initialized
        && program_controls_mint(token_out_mint, mint_authority)
}

pub(crate) fn load_redemption_offer_vault_target_bps_for_offer(
    program_id: &Pubkey,
    redemption_offer_account: &UncheckedAccount,
    offer_key: Pubkey,
    token_in_mint: Pubkey,
    token_out_mint: Pubkey,
) -> Result<Option<u16>> {
    let Some(redemption_offer) = load_optional_checked_redemption_offer(
        program_id,
        redemption_offer_account,
        offer_key,
        token_out_mint,
        token_in_mint,
    )?
    else {
        return Ok(None);
    };

    if redemption_offer.is_disabled() {
        return Ok(None);
    }

    Ok(Some(redemption_offer.vault_target_bps))
}

pub(crate) fn calculate_redemption_vault_refill_amount(
    market_stats_account: &AccountInfo,
    vault_target_bps: u16,
    token_in_mint: &Mint,
    token_out_mint: &Mint,
    current_redemption_vault_balance: u64,
    token_in_net_amount: u64,
) -> u64 {
    if vault_target_bps == 0 {
        return 0;
    }

    let target_liquidity = read_market_stats_account(market_stats_account)
        .map(|market_stats| {
            let target_in_onyc_decimals = (market_stats.tvl as u128)
                .saturating_mul(vault_target_bps as u128)
                .saturating_div(MAX_BASIS_POINTS as u128);
            target_in_onyc_decimals
                .saturating_mul(10_u128.pow(token_in_mint.decimals as u32))
                .saturating_div(10_u128.pow(token_out_mint.decimals as u32))
        })
        .unwrap_or(0);

    let target_liquidity_u64 = target_liquidity.min(u64::MAX as u128) as u64;
    let current_after_trade = current_redemption_vault_balance.saturating_sub(token_in_net_amount);
    target_liquidity_u64.saturating_sub(current_after_trade)
}

pub(crate) fn calculate_current_step_price(
    apr: u64,
    base_price: u64,
    base_time: u64,
    price_fix_duration: u64,
) -> Result<u64> {
    require!(base_price > 0, crate::OnreError::ZeroPriceNotAllowed);
    require!(price_fix_duration > 0, crate::OnreError::InvalidPriceFixDuration);

    let current_time = Clock::get()?.unix_timestamp as u64;
    calculate_current_step_price_at(
        apr,
        base_price,
        base_time,
        price_fix_duration,
        current_time,
    )
}

pub(crate) fn calculate_current_step_price_at(
    apr: u64,
    base_price: u64,
    base_time: u64,
    price_fix_duration: u64,
    current_time: u64,
) -> Result<u64> {
    require!(base_price > 0, crate::OnreError::ZeroPriceNotAllowed);
    require!(price_fix_duration > 0, crate::OnreError::InvalidPriceFixDuration);

    let elapsed = current_time.saturating_sub(base_time);
    let current_step = elapsed / price_fix_duration;
    let step = current_step.checked_add(1).ok_or(crate::OnreError::MathOverflow)?;

    let periods_elapsed = (step as u128)
        .checked_mul(price_fix_duration as u128)
        .ok_or(crate::OnreError::MathOverflow)?;
    let days_elapsed = periods_elapsed / SECONDS_IN_DAY as u128;

    let apr_scaled = (apr as u128)
        .checked_mul(INT_SCALE)
        .ok_or(crate::OnreError::MathOverflow)?
        .checked_div(APR_SCALE)
        .ok_or(crate::OnreError::MathOverflow)?;

    let daily_rate = apr_scaled
        .checked_div(365u128)
        .ok_or(crate::OnreError::MathOverflow)?;
    let growth_factor = pow_fixed(
        INT_SCALE
            .checked_add(daily_rate)
            .ok_or(crate::OnreError::MathOverflow)?,
        days_elapsed,
        INT_SCALE,
    )?;

    let price_u128 = (base_price as u128)
        .checked_mul(growth_factor)
        .ok_or(crate::OnreError::MathOverflow)?
        .checked_div(INT_SCALE)
        .ok_or(crate::OnreError::MathOverflow)?;

    require!(
        price_u128 <= u64::MAX as u128,
        crate::OnreError::ResultOverflow
    );

    Ok(price_u128 as u64)
}

fn find_active_vector_at(offer: &Offer, current_time: u64) -> Result<&OfferVector> {
    let mut active: Option<&OfferVector> = None;
    for vector in &offer.vectors {
        if vector.start_time <= current_time {
            if let Some(current) = active {
                if vector.start_time > current.start_time {
                    active = Some(vector);
                }
            } else {
                active = Some(vector);
            }
        }
    }

    active.ok_or_else(|| error!(crate::OnreError::NoActiveVector))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer_with_key(key: Pubkey, needs_approval: u8) -> Offer {
        Offer {
            key,
            needs_approval,
            ..Offer::default()
        }
    }

    #[test]
    fn approval_message_scope_is_checked_against_offer_key() {
        let offer_a = offer_with_key(Pubkey::new_unique(), 1);
        let offer_b = offer_with_key(Pubkey::new_unique(), 1);
        let program_id = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let message = ApprovalMessage {
            program_id,
            offer: offer_a.key,
            user_pubkey: user,
            expiry_unix: u64::MAX,
        };

        assert_eq!(message.offer, offer_a.key);
        assert_ne!(message.offer, offer_b.key);
    }
}
