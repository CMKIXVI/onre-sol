# BUFFER Operations Runbook

## Preconditions

Before initializing BUFFER, the canonical ONyc offer must exist and `state.main_offer` must point to an offer whose `token_out_mint` is ONyc.

Recommended setup order:

1. Create the ONyc offer.
2. Set `state.main_offer`.
3. Initialize BUFFER.
4. Set the gross APR.
5. Configure fee/high-water-mark behavior if required.

## Accrual inputs

BUFFER accrual uses two separate inputs:

- `gross_apr`: management configuration, capped by the program.
- `current_yield`: derived from the active APR on `state.main_offer`.

The audited hardening rejects an invalid ordering where current yield exceeds gross yield instead of silently converting the negative delta to zero.

## Reserve movement

Reserve deposits and withdrawals move reserve-token balances. They do not themselves change ONyc supply. Supply-changing operations have separate accounting and baseline requirements.

## Operational checks

- Confirm `state.main_offer` before BUFFER initialization or supply changes.
- Confirm the active offer has ONyc as its output mint.
- Treat an invalid APR ordering as a configuration error requiring correction, not as zero yield.
- Preserve canonical reserve/configurable-vault account derivations.
- Keep the kill switch state visible to operators before value-moving actions.
- Reconcile fees and reserve balances after settlement operations.

## Verification note

This runbook captures source-level invariants reviewed during the Aetheron audit. Runtime tests/builds were not executed: `NOT RUN — environment/toolchain not assumed available`.
