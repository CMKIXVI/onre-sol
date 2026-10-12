# Security Audit Invariants

This document records the source-level invariants reviewed during the OneRe security audit. It is intended as a maintainer review checklist for future changes.

## Authority

- Boss transfer is two-step: proposal by the current boss and acceptance by the proposed boss.
- Worker assignment is boss-authorized.
- Kill-switch enablement may be performed by the configured emergency authorities; disabling remains boss-controlled.
- Targeted offer re-enable remains boss-controlled.

## Offers

- Offer vaults are controlled by canonical program-derived authorities.
- Offer execution must use the intended offer and configured mint pair.
- Approval-gated offers must authenticate the exact offer being executed.
- Permissionless offer paths intentionally do not require regular approval signatures.

## Redemption

- Redemption requests are bound to redemption offer, redeemer, and request ID.
- Fulfillment must remain bound to the request and its redeemer/offer identity.
- Fulfillment amount cannot exceed the request's remaining amount.
- Worker authorization is required for worker settlement paths.

## BUFFER

- Accrual uses the configured main offer as the canonical yield/NAV source.
- Gross APR must not be lower than the current yield used for accrual calculation.
- Fee basis points are capped at configuration time.
- Accrual arithmetic uses checked u128 intermediates and explicit u64 result bounds.
- Reserve withdrawal does not mutate ONyc supply and therefore does not require a supply-baseline reset.

## Supply and NAV

- ONyc minting and burning are bounded by configured supply/mint limits.
- Circulating supply excludes the canonical program-owned excluded balance in V2 market accounting.
- NAV/TVL calculations use checked arithmetic.
- Legacy market views retain their historical account contracts; V2 views are the preferred current accounting interface.

## Token programs and accounts

- Canonical ATAs are derived from the expected owner, mint, and token program.
- Guarded value-moving paths reject unsupported Token-2022 transfer-fee configurations.
- ONyc redemption input/request creation uses the legacy SPL Token program where required by the accounting model.
- Token-program substitution and arbitrary account aliasing must not bypass these constraints.

## Verification policy

This audit was source-level. Runtime tests/builds were intentionally not executed because the environment/toolchain was not assumed available. Future reviewers should run the repository's normal LiteSVM/integration suite before merging changes that affect these invariants.
