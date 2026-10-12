# Token Program Compatibility

## Purpose

Onre supports SPL Token and selected Token-2022 paths, but value-moving instructions cannot safely account for arbitrary Token-2022 transfer-fee deltas. The program therefore rejects Token-2022 mints with non-zero transfer fees on guarded paths where the expected amount must equal the transferred amount.

## Guarded paths

Do not integrate a non-zero-transfer-fee Token-2022 mint with:

- offer execution
- redemption operations
- offer/redemption vault deposits and withdrawals
- Prop AMM quotes or execution
- BUFFER reserve movement
- configurable-vault withdrawals

These paths must preserve explicit token-account and accounting invariants, so silently accepting a transfer-fee deduction would create an amount mismatch.

## Integration rule

Before selecting a Token-2022 mint for a guarded path, verify that its transfer-fee configuration is compatible with the instruction. A mint that can impose a non-zero transfer fee should be treated as unsupported for these flows unless the program explicitly documents support for that mint configuration.

## Why this is intentional

The audit reviewed token-program selection, ATA derivation, transfer helpers, and Token-2022 handling. The relevant paths already reject unsupported transfer-fee configurations rather than attempting to infer the post-transfer amount. This document makes that boundary explicit for integrators and helps prevent clients from constructing transactions that are expected to fail.

## Related invariants

- Canonical ATAs are derived from the expected mint, owner and token program.
- Redemption ONyc input/request creation requires the legacy SPL Token program.
- Mint/burn/transfer helpers validate the token program and reject unsupported transfer-fee behavior.
- Account substitution and token-program confusion must not be used as a workaround for the transfer-fee restriction.
