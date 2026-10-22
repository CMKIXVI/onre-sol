# OnRe Solana — Complete Investigation Report

**Repository under review:** `onre-finance/onre-sol`  
**Personal working fork:** `CMKIXVI/onre-sol`  
**Current upstream master reviewed:** `c0490831f438074562ebf51e437ca5f8e027596b`  
**Report classification:** authorized security investigation record / hypothesis-elimination report  
**Conclusion:** **No confirmed bounty-grade vulnerability established**

## 1. Executive summary

This report consolidates the full OnRe Solana investigation performed in the authorized bounty lane. The work was deliberately focused on the strongest remaining state-transition and accounting chains rather than broad scanning.

The investigation covered:

1. permissionless V2 → BUFFER/main-offer accrual → redemption-vault refill → MarketStats;
2. redemption create → partial fulfill → cancel/recreate → settlement;
3. BUFFER/max-supply boundary → redemption;
4. Prop-AMM sell → redemption liquidity → accounting;
5. cached excluded balance → MarketStats → downstream pricing;
6. redemption replay/double-use and request-PDA substitution;
7. authorization and worker/redeemer boundary checks;
8. canonical `main_offer` versus alternate-offer/global-BUFFER pricing assumptions;
9. MarketStats freshness, refresh ordering, timestamps/slots, and excluded-balance cache behavior;
10. redemption offer/vault refill and settlement behavior;
11. deployment/artifact correlation and whether candidate behavior could be tied to the latest deployed artifact;
12. existing repository fuzz/invariant infrastructure and open PRs relevant to the analysis.

The investigation did identify a technically reproducible **Prop-AMM cadence-state griefing behavior** in a local/third-party harness: repeated dust sells can increase `curr_sell_trade_count` and worsen a later victim quote. The measured scenario required approximately 19 additional dust sells of 875 ONyc base units each to saturate the cadence threshold. However, the resulting impact was a liquidity/DoS-style degradation, which is explicitly outside the relevant bounty impact scope. In addition, the evidence available did not establish cryptographic equivalence between that tested state and the latest deployed artifact. It was therefore **not submitted as a bounty finding**.

The other principal chains were falsified, bounded by existing protections, or reduced to temporary/stale-state effects without an eligible security impact. No deterministic runnable PoC demonstrating an eligible bounty impact on the latest deployed artifact was established.

The final deliverable therefore records the investigation and preserves a deterministic redemption accounting regression test, without claiming a vulnerability, acceptance, payment, or bounty win.

---

## 2. Scope and methodology

The investigation followed the authorized OnRe Solana bounty scope and safe local-testing constraints. The objective was not to maximize the number of hypotheses, but to attack the strongest remaining chains and eliminate them quickly when protections held.

For each candidate chain the analysis used the following sequence:

1. identify attacker-controlled inputs and the smallest possible transaction/state sequence;
2. identify the accounting or authorization invariant that would have to fail;
3. inspect the relevant on-chain instruction/account constraints;
4. attempt deterministic reproduction where practical;
5. determine whether the observed behavior has an eligible security impact;
6. check whether the behavior can be correlated to the deployed artifact;
7. kill the hypothesis if any required condition fails.

A candidate was not treated as confirmed merely because unusual state behavior existed. Confirmation required a reproducible security impact, bounty eligibility, and sufficient correlation to the relevant deployed artifact.

No unauthorized scanning, credential abuse, persistence, destructive action, DDoS/spam, social engineering, or scope bypass was used.

---

## 3. Repository/deployment state

### Current master

The current upstream master reviewed during the investigation was:

`c0490831f438074562ebf51e437ca5f8e027596b`

### Relevant deployed-artifact evidence

A previously identified deployment correlation pointed to commit:

`1dbc6ec0ed18a8fc81516f930d85e669b689d852`

The investigation did **not** establish cryptographic equivalence between that deployment evidence and current master. This distinction is important: a behavior found in source or a harness cannot automatically be represented as a deployed-program vulnerability.

### Relevant live program

The Prop-AMM investigation correlated the examined live program with:

`onreuGhHHgVzMWSkj2oQDLDtvvGvoepBPkqyaubFcwe`

The existence of the live program was not treated as proof that every current-master behavior was deployed there.

---

## 4. Primary chain A — Permissionless V2 → BUFFER/main-offer accrual → redemption-vault refill → MarketStats

### Hypothesis

A permissionless V2 operation could change BUFFER/main-offer state, after which redemption-vault refill and/or MarketStats could observe inconsistent state and expose a pricing or accounting discrepancy to an attacker or victim.

### Investigation

The chain was examined from the state transition through the downstream redemption and MarketStats calculations. Particular attention was given to the ordering between:

- V2 offer activity;
- BUFFER/global liquidity state;
- the canonical `main_offer`;
- redemption-vault target/refill behavior;
- MarketStats refresh;
- cached circulating-supply exclusions.

### Result

The observed effect was a **state-freshness/ordering lag**, rather than a demonstrated unauthorized extraction or eligible loss. In the investigated path, MarketStats could reflect pre-refresh state until the permissionless refresh operation was performed.

The resulting target-liquidity lag did not establish an eligible bounty impact. The investigation therefore killed this chain as a confirmed vulnerability.

**Disposition: ELIMINATED / no eligible bounty impact demonstrated.**

---

## 5. Primary chain B — Redemption create → partial fulfill → cancel/recreate → settlement

### Hypothesis

A redemption request might be partially fulfilled and then cancelled in a way that:

- returned already-fulfilled tokens;
- left stale redemption liability;
- double-counted settled proceeds;
- allowed request reuse;
- caused vault/liability divergence;
- or made a subsequent redemption inherit inconsistent accounting.

### Lifecycle tested

`create redemption → partial fulfill → cancel remaining → create another redemption → fulfill/settle`

### Invariants

For total created redemption amount `C`:

`C = V + S + R`

where:

- `V` = amount still locked against pending redemption requests;
- `S` = amount fulfilled/settled;
- `R` = amount returned through cancellation.

The pending-liability invariant is:

`Q = V`

where `Q` is the remaining requested redemption liability.

For a request of original amount `A`, partially fulfilled by `F`, the cancellable amount must be:

`A - F`

not `A`.

### Existing protections examined

The redemption cancellation path uses request/offer PDA binding and explicit offer matching. Signer authorization is constrained to the configured authorized worker/boss or original redeemer according to the current account definition. The request account is closed after cancellation.

The investigation also considered replay/double-use and request-PDA substitution. No reproducible path was established that bypassed the request binding or authorization boundaries.

### Deterministic regression model

A standalone deterministic accounting model was added to the personal fork. It covers:

- create → partial fulfill → cancel;
- multiple independent requests with global reconciliation;
- cancellation returning only the remaining unfulfilled amount.

The model checks conservation after every state transition.

### Result

No accounting break was reproduced. The chain remains a useful regression boundary, but it is **not a vulnerability finding**.

**Disposition: ELIMINATED / regression coverage added.**

---

## 6. Primary chain C — BUFFER/max-supply boundary → redemption

### Hypothesis

A boundary interaction between BUFFER accounting, maximum supply constraints, and redemption could permit a state transition that bypasses intended supply/liquidity controls or produces an accounting imbalance.

### Investigation

The analysis examined the boundary behavior around configured maximum supply and the downstream redemption path, including whether a permissionless operation could create an inconsistent state that redemption would accept.

### Result

No reproducible eligible bypass or extraction path was established. The relevant state constraints and accounting boundaries did not yield a bounty-grade survivor.

**Disposition: ELIMINATED.**

---

## 7. Primary chain D — Prop-AMM sell → redemption liquidity → accounting

### Hypothesis

A third party could perform low-value Prop-AMM sells to alter internal cadence/liquidity state and thereby materially worsen a later redemption or victim transaction.

### Observed mechanism

The Prop-AMM path increments `curr_sell_trade_count` on successful sells and uses the cadence state in the haircut/quote calculation.

A deterministic third-party harness produced a state-griefing sequence in which repeated dust sells could saturate the cadence threshold and worsen a later victim quote.

The measured scenario was approximately:

- **19** additional dust sells;
- **875 ONyc base units per sell**;
- resulting in saturation/worsening of the later quote.

### Security assessment

The behavior is real as a state/cadence effect in the tested model, but the resulting impact was characterized as liquidity/DoS-style degradation. The relevant bounty rules exclude lack-of-liquidity style impact.

In addition, the available evidence did not establish cryptographic equivalence between the tested source/deployment state and the latest deployed artifact.

### Result

This was the strongest non-trivial candidate found, but it does **not** satisfy the confirmation standard for the bounty.

**Disposition: ELIMINATED AS BOUNTY FINDING; preserve as an excluded-impact hypothesis/evidence item.**

---

## 8. Primary chain E — Cached excluded balance → MarketStats → downstream pricing

### Hypothesis

A stale circulating-supply-excluded-balance cache could be used to make MarketStats reflect materially incorrect supply/NAV-related values and influence downstream pricing.

### Relevant architecture

Current master retains the MarketStats refresh path in `programs/onreapp/src/instructions/market_info/refresh_market_stats.rs`.

The refresh path:

- validates the canonical `main_offer` against `state.main_offer`;
- validates token-in and ONyc mints;
- loads the excluded-balance PDA;
- treats an uninitialized excluded-balance PDA as zero;
- recomputes MarketStats;
- updates the MarketStats PDA;
- emits a refresh event.

The excluded balance is intentionally cached. When excluded balances or the configured owner list changes, the excluded-balance cache is expected to be updated before MarketStats is refreshed.

### Investigation

The analysis checked stale timestamps/slots, refresh ordering, uninitialized-cache behavior, and downstream use of the MarketStats snapshot.

### Result

The identified behavior is a freshness/order dependency, not a demonstrated unauthorized accounting exploit. The V6 work in PR #153 further documents the intended snapshot semantics and refresh ordering.

No bounty-grade extraction, privilege bypass, or eligible financial impact was reproduced.

**Disposition: ELIMINATED / known freshness semantics.**

---

## 9. Additional hypothesis — Redemption replay / double-use

The investigation separately considered whether an already-used redemption request or partially processed request could be replayed to obtain a second settlement.

The request lifecycle, account closure, request/offer binding, and fulfillment/cancellation state transitions were inspected. No reproducible double-settlement sequence survived the state/account constraints.

**Disposition: ELIMINATED.**

---

## 10. Additional hypothesis — PDA/account substitution

The investigation checked whether attacker-controlled accounts could be substituted for expected redemption offers, requests, mints, state accounts, or MarketStats-related accounts.

The relevant PDA seeds, account constraints, and `has_one`/mint relationships were considered. No reproducible unauthorized substitution path was established.

**Disposition: ELIMINATED.**

---

## 11. Additional hypothesis — Authorization boundary / worker versus redeemer

The redemption lifecycle was examined for confused-deputy behavior between the configured worker/boss authority and the original redeemer.

No sequence was established in which an unauthorized actor could use the lifecycle permissions to obtain an eligible financial advantage.

**Disposition: ELIMINATED.**

---

## 12. Additional hypothesis — Canonical main_offer versus alternate-offer/global BUFFER pricing

The investigation checked whether state derived from the canonical `main_offer` could be confused with alternate-offer or global BUFFER state, creating a mismatch exploitable through pricing or redemption.

The analysis did not produce a reproducible eligible extraction or accounting break. The canonical-state checks and explicit main-offer relationship were treated as protections where applicable.

**Disposition: ELIMINATED.**

---

## 13. MarketStats / Program V6 review

Open PR **#153 — Program V6** was reviewed because it materially changes the MarketInfo architecture.

Relevant changes include:

- removal of V1 offer instructions;
- removal of several legacy MarketInfo getters;
- canonical market metrics moving to the `MarketStats` PDA;
- MarketStats as a cached snapshot of `state.main_offer`;
- documented `lastUpdatedAt` and `lastUpdatedSlot` freshness fields;
- permissionless refresh, with the signer paying first-PDA creation;
- explicit excluded-balance refresh ordering;
- V2 redemption-related instructions remaining in the program.

This was treated as repository architecture and state-model evidence, not as a vulnerability itself.

**Disposition: architecture reviewed; no bounty finding.**

---

## 14. Existing fuzz/invariant infrastructure — PR #149

Open PR **#149 — “fuzz: crucible SVM invariant harness for onreapp + FuzzCorp CI”** was reviewed as relevant existing infrastructure.

The PR is additive and introduces Crucible/LiteSVM invariant testing, builds `onreapp` from source, asserts invariants after instructions, and integrates with FuzzCorp. It requires `FUZZ_API_KEY` for its external workflow.

The investigation deliberately did not misrepresent this infrastructure as a completed exploit proof. It was treated as supporting test infrastructure and evidence of existing invariant-oriented development.

**Disposition: supporting infrastructure; no independent vulnerability.**

---

## 15. Evidence and reproducibility status

### What was reproduced

- deterministic accounting relationships for the redemption lifecycle in a local model;
- the Prop-AMM cadence-state griefing behavior in the reported harness conditions;
- repository/source-level state and account relationships relevant to the investigated chains.

### What was not established

A deterministic, runnable exploit PoC satisfying all of the following simultaneously was not established:

1. attacker-controlled sequence;
2. eligible bounty impact;
3. current deployed-artifact equivalence;
4. reproducible end-to-end proof on the relevant deployed state.

That is the decisive reason no bounty finding is claimed.

---

## 16. Final disposition by hypothesis

| Investigation | Result | Bounty status |
|---|---|---|
| V2 → BUFFER → redemption refill → MarketStats | Temporary/freshness behavior only | Not eligible / eliminated |
| Redemption partial fulfill → cancel → recreate → settlement | Accounting conserved | No finding |
| BUFFER/max-supply → redemption | No bypass reproduced | No finding |
| Prop-AMM sell → redemption liquidity | Cadence griefing reproduced, but impact excluded | Not bounty-eligible |
| Cached excluded balance → MarketStats | Staleness/order dependency | No finding |
| Redemption replay/double-use | No surviving replay path | Eliminated |
| PDA/account substitution | No unauthorized substitution | Eliminated |
| Worker/redeemer authorization | No privilege bypass | Eliminated |
| main_offer/global BUFFER/alternate-offer pricing | No eligible mismatch exploit | Eliminated |
| Deployment correlation | Latest artifact equivalence not proven | Blocker to confirmation |

---

## 17. Deterministic redemption regression artifact

The personal fork contains:

`programs/onreapp/tests/redemption_settlement_invariants.rs`

It defines a small deterministic redemption ledger with `create`, `fulfill`, `cancel`, and conservation checks. The tests cover:

1. `create_partial_fulfill_cancel_reconciles`;
2. `multiple_requests_reconcile_independently_and_globally`;
3. `cancellation_returns_only_the_remaining_request`.

The test is intentionally production-neutral. It does not modify on-chain program behavior and does not claim that a vulnerability was found.

---

## 18. Personal-fork evidence

The investigation artifacts were preserved in the personal fork:

`CMKIXVI/onre-sol`

Relevant branch:

`test/redemption-settlement-invariants`

Redemption invariant test commit:

`65da4b8055b26b9be806d1c1dcfcf69bd2c7921e`

The accompanying focused redemption report was also created at:

`reports/redemption-settlement-invariant-report.md`

An attempted upstream PR creation was blocked by the GitHub integration permission boundary (`403 Resource not accessible by integration`). Therefore, no upstream PR submission is claimed here.

---

## 19. Overall conclusion

**NO CONFIRMED BOUNTY-GRADE FINDING.**

The investigation was substantive and included multiple independent attack chains, state-machine analysis, authorization checks, accounting invariants, MarketStats freshness analysis, Prop-AMM cadence analysis, and deployment-correlation review.

The absence of a confirmed finding is not based on stopping after a superficial source review. The strongest candidate that produced a measurable adverse effect was the Prop-AMM cadence-state griefing path, but its impact was outside the bounty's eligible impact class and its latest-deployment equivalence was not established.

The redemption lifecycle was converted into an explicit deterministic accounting regression boundary. This preserves the most important state-machine invariant discovered during the investigation without making an unsupported security claim.

**Final security status:** no bounty submission warranted on the evidence presently available.

**Final evidence blocker:** current deployed-artifact correlation and an eligible, deterministic end-to-end exploit PoC were not both established for any surviving hypothesis.
