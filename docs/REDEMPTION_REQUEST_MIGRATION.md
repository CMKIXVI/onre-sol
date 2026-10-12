# Redemption Request Migration Guide

## Request identity

`create_redemption_request` uses a frontend-generated request ID of exactly 32 UTF-8 bytes. The request PDA is derived from:

- the `redemption_request` seed,
- the redemption-offer PDA,
- the redeemer public key, and
- the request ID bytes.

Persist both the request ID and resulting PDA in the backend so workers can fulfill the exact request and users can cancel the exact request.

## Upgrade checklist

1. Generate a unique 32-byte request ID for every new request.
2. Persist the request ID before handing the request to the worker flow.
3. Persist the derived request PDA alongside the ID and redeemer.
4. Resolve pending requests before upgrading across the account-layout/instruction change.
5. Ensure fulfillment uses the same redemption offer, redeemer identity, request PDA, and request ID that were created together.
6. Do not reuse a request ID across redemption offers or users.

## Why this matters

The request identity is intentionally bound to both the redemption offer and redeemer. Backend systems that treat the request ID as a globally sufficient identifier can accidentally address the wrong application state or fail to locate the canonical PDA.

## Verification note

This guide reflects source-level audit review. Runtime tests were not executed: `NOT RUN — environment/toolchain not assumed available`.
