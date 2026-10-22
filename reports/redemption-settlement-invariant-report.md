# Redemption Settlement Accounting Invariant Report

**Repository:** `CMKIXVI/onre-sol`  
**Working branch:** `test/redemption-settlement-invariants`  
**Scope:** deterministic redemption lifecycle accounting review  
**Classification:** regression/invariant coverage; **not a vulnerability report**

## 1. Executive summary

This report records the deterministic state-machine analysis performed on the redemption lifecycle and the accompanying regression tests.

The investigated lifecycle is:

`create redemption -> partial fulfill -> cancel remaining -> create another redemption -> fulfill/settle`

The objective was to make the accounting contract explicit and test the conservation relationships across each state transition. The review did **not** reproduce an accounting break or establish a bounty-grade vulnerability.

The resulting test suite is intentionally production-neutral: it adds accounting invariants and does not modify on-chain program behavior.

## 2. Security question

The central question was whether a redemption request could transition through partial fulfillment and cancellation in a way that causes tokens, redemption liabilities, settled amounts, or returned amounts to become inconsistent.

The specific properties examined were:

1. A created redemption amount must remain fully accounted for.
2. A partial fulfillment may remove only the fulfilled amount from the pending liability.
3. Cancellation after partial fulfillment may return only the unfulfilled remainder.
4. Pending redemption liability must reconcile with the amount still represented as locked in the ledger model.
5. Multiple simultaneous requests must remain globally conserved.

## 3. Invariant model

For a total amount `C` created across redemption requests, define:

- `V` = token amount still locked against pending redemption requests
- `S` = token amount already fulfilled/settled
- `R` = token amount returned through cancellation
- `Q` = remaining requested redemption liability

The primary conservation invariant is:

`C = V + S + R`

The pending-liability invariant is:

`Q = V`

These relationships must hold after every lifecycle transition, not only at the end of the scenario.

## 4. State transitions

### Creation

For a request of amount `A`:

`Q += A`

`V += A`

No amount is settled or returned.

### Partial fulfillment

For fulfillment amount `F`:

`F <= Q`

`F <= V`

Then:

`Q -= F`

`V -= F`

`S += F`

### Cancellation

For cancellation amount `K`:

`K <= Q`

`K <= V`

Then:

`Q -= K`

`V -= K`

`R += K`

For a partially fulfilled request with original amount `A` and fulfilled amount `F`, the cancellable remainder is:

`K = A - F`

Returning the original amount after partial fulfillment would violate conservation; the regression test explicitly guards against that class of error.

## 5. Deterministic test coverage

The committed test file `programs/onreapp/tests/redemption_settlement_invariants.rs` contains three deterministic scenarios.

### Test 1 — create, partial fulfill, cancel

A request of `1,000` is created, `400` is fulfilled, and the remaining `600` is cancelled.

Expected final state:

- requested = `0`
- vault/locked amount = `0`
- settled = `400`
- returned = `600`
- total conserved = `1,000`

The invariant is checked after creation, after fulfillment, and after cancellation.

### Test 2 — multiple requests

Two requests of `1,000` and `750` are created and then processed through a mixture of fulfillment and cancellation.

The test checks conservation after every transition and verifies that independent request activity remains globally reconcilable.

Expected final accounting:

- requested = `400`
- vault/locked amount = `400`
- settled = `850`
- returned = `500`
- total created = `1,750`

### Test 3 — cancellation after partial fulfillment

A `900` request is partially fulfilled by `350` and then cancelled.

The test explicitly derives the remaining cancellable amount as:

`900 - 350 = 550`

It verifies that only `550` is returned and that the `350` already settled is not returned again.

## 6. Existing program controls considered

The current redemption cancellation implementation binds the redemption request to the redemption offer through the request PDA seeds and an explicit offer-match constraint. The cancellation signer is constrained to the configured boss, worker, or the original redeemer. fileciteturn41file0

The request account is closed after cancellation, with rent returned to the worker according to the current account definition. fileciteturn41file0

These controls were treated as existing protections rather than newly discovered issues.

## 7. Result

**Result: no confirmed accounting vulnerability.**

The deterministic invariant model remained conserved for the tested lifecycle sequences. The work therefore provides regression protection rather than a security finding.

No claim is made here about deployed-program equivalence, bounty eligibility, acceptance, or payment.

## 8. Security impact classification

This report should **not** be classified as a vulnerability submission by itself.

It is best classified as:

- regression test
- accounting invariant coverage
- state-machine validation
- future-change protection

Production behavior is unchanged by the test-only addition.

## 9. Relationship to other work

This report is deliberately narrower than the broader fuzzing/invariant infrastructure and separate from the V6/MarketStats architectural changes. It does not duplicate those efforts and does not alter production instructions.

The purpose is to preserve the exact accounting contract identified during the redemption investigation in a small deterministic regression suite.

## 10. Recommended disposition

Keep the invariant tests with the redemption test suite and run them as part of the normal Rust/LiteSVM test workflow.

If future production changes modify redemption accounting, the tests should fail at the first transition where conservation is broken, making the regression boundary explicit.

## 11. Evidence

Working branch:

`test/redemption-settlement-invariants`

Test commit:

`65da4b8055b26b9be806d1c1dcfcf69bd2c7921e`

The test file contains the executable ledger model and the three deterministic scenarios described above.
