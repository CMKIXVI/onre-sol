# Kill-Switch Operation Matrix

## Purpose

The kill switch is an emergency stop for guarded value-moving paths. It is not a global program pause. Operators and integrators must distinguish emergency-stop behavior from governance/configuration behavior.

## Guarded while `state.is_killed == true`

| Area | Instructions |
|---|---|
| Offer execution | `take_offer`, `take_offer_v2`, `take_offer_permissionless`, `take_offer_permissionless_v2` |
| Prop AMM | `quote_swap_buy`, `quote_swap_sell`, `open_swap_buy`, `open_swap_sell` |
| Redemption movement | `create_redemption_request`, `fulfill_redemption_request`, `cancel_redemption_request` |
| Offer/redemption vault movement | `offer_vault_deposit`, `offer_vault_withdraw`, `redemption_vault_deposit`, `redemption_vault_withdraw` |
| BUFFER/configurable vault movement | `settle_buffer`, `deposit_reserve_vault`, `withdraw_reserve_vault`, `withdraw_configurable_vault` |
| Supply/accrual movement | `mint_to`, `burn_for_nav_increase`, `set_buffer_gross_apr`, `set_buffer_fee_config` |

## Not globally paused

Governance and configuration operations can remain callable according to their own authorization rules. Examples include authority changes, admin/approver management, offer configuration, targeted disable, supply-cap configuration, and mint-authority transfer.

## Operator guidance

1. Treat enabling the kill switch as an emergency containment action.
2. Do not assume every instruction is paused after the switch is enabled.
3. Verify the caller's role before attempting to disable the switch; admins can enable but only the boss can disable it.
4. Reconcile pending redemptions and operational state before resuming guarded flows.
5. Integrations should surface the killed state before presenting value-moving actions to users.

## Review note

This document records the source-level behavior reviewed during the Aetheron security audit. Runtime tests were not executed: `NOT RUN — environment/toolchain not assumed available`.
