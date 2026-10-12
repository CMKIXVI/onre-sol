use crate::utils::approver::message::ApprovalMessage;
use crate::utils::ed25519_parser::parse_ed25519_ix;
use anchor_lang::prelude::*;
use solana_instructions_sysvar::{load_current_index_checked, load_instruction_at_checked};
use solana_program::ed25519_program;

/// Verifies cryptographic approval messages signed by trusted authorities.
///
/// Approval messages are domain-separated by program, offer, and user. This prevents
/// a signature issued for one approval-gated offer from being replayed against another
/// offer for the same user before the signature expires.
pub fn verify_approval_message_generic(
    program_id: &Pubkey,
    offer_key: &Pubkey,
    user_pubkey: &Pubkey,
    approver1: &Pubkey,
    approver2: &Pubkey,
    instructions_sysvar: &UncheckedAccount,
    msg: &ApprovalMessage,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp as u64;
    validate_approval_message_context(program_id, offer_key, user_pubkey, now, msg)?;

    // Find the previous instruction and ensure it is an Ed25519 verify instruction.
    let cur_idx = load_current_index_checked(&instructions_sysvar.to_account_info())
        .map_err(|_| crate::OnreError::MissingEd25519Ix)?;
    require!(cur_idx > 0, crate::OnreError::MissingEd25519Ix);

    let ix = load_instruction_at_checked(
        (cur_idx - 1) as usize,
        &instructions_sysvar.to_account_info(),
    )
    .map_err(|_| crate::OnreError::MissingEd25519Ix)?;

    require!(
        ix.program_id == ed25519_program::id(),
        crate::OnreError::WrongIxProgram
    );
    require!(ix.accounts.is_empty(), crate::OnreError::BadEd25519Accounts);

    let parsed = parse_ed25519_ix(&ix.data).ok_or(crate::OnreError::MalformedEd25519Ix)?;
    require!(parsed.sig_count == 1, crate::OnreError::MultipleSigs);

    let is_approver1 = *approver1 != Pubkey::default() && parsed.pubkey == approver1.to_bytes();
    let is_approver2 = *approver2 != Pubkey::default() && parsed.pubkey == approver2.to_bytes();
    require!(
        is_approver1 || is_approver2,
        crate::OnreError::WrongAuthority
    );

    let signed_msg = ApprovalMessage::try_from_slice(&parsed.message)
        .map_err(|_| crate::OnreError::MsgDeserialize)?;
    validate_approval_message_context(program_id, offer_key, user_pubkey, now, &signed_msg)?;
    require!(signed_msg == *msg, crate::OnreError::MsgMismatch);

    Ok(())
}

fn validate_approval_message_context(
    program_id: &Pubkey,
    offer_key: &Pubkey,
    user_pubkey: &Pubkey,
    now: u64,
    msg: &ApprovalMessage,
) -> Result<()> {
    require!(now <= msg.expiry_unix, crate::OnreError::Expired);
    require!(msg.program_id == *program_id, crate::OnreError::WrongProgram);
    require!(msg.offer == *offer_key, crate::OnreError::WrongOffer);
    require!(msg.user_pubkey == *user_pubkey, crate::OnreError::WrongUser);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_context_is_bound_to_offer() {
        let program_id = Pubkey::new_unique();
        let offer_a = Pubkey::new_unique();
        let offer_b = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let msg = ApprovalMessage {
            program_id,
            offer: offer_a,
            user_pubkey: user,
            expiry_unix: 100,
        };

        assert!(validate_approval_message_context(&program_id, &offer_a, &user, 50, &msg).is_ok());
        assert!(validate_approval_message_context(&program_id, &offer_b, &user, 50, &msg).is_err());
    }

    #[test]
    fn approval_context_rejects_expired_message() {
        let program_id = Pubkey::new_unique();
        let offer = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let msg = ApprovalMessage {
            program_id,
            offer,
            user_pubkey: user,
            expiry_unix: 100,
        };

        assert!(validate_approval_message_context(&program_id, &offer, &user, 101, &msg).is_err());
    }
}
