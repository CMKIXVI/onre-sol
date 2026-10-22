# OnRe Solana — Complete Investigation Report v2

**Repository:** `onre-finance/onre-sol`  
**Personal fork:** `CMKIXVI/onre-sol`  
**Investigation branch:** `reports/complete-onre-investigation`  
**Current upstream master reviewed:** `c0490831f438074562ebf51e437ca5f8e027596b`  
**Relevant historical/deployment correlation reviewed:** `1dbc6ec0ed18a8fc81516f930d85e669b689d852`  
**Report type:** comprehensive authorized security investigation record  
**Security conclusion:** **No confirmed bounty-grade vulnerability established**

---

# 1. Purpose of this report

This document is the consolidated record of the OnRe Solana investigation. It is intentionally much broader than the final redemption regression-test note.

The purpose is to preserve the reasoning path, hypotheses, state transitions, protections, reproduction results, exclusions, deployment questions, repository evidence, and final disposition in one place.

This is an investigation report, not a claim that a vulnerability was found. A hypothesis appearing in this report does not mean it survived analysis. Where a behavior was observed but failed the bounty-impact or deployment-correlation test, that distinction is recorded explicitly.

The central standard used throughout was:

> A behavior is not a confirmed security finding merely because it is unusual, theoretically exploitable, or locally reproducible. It must produce an eligible security impact and be sufficiently tied to the relevant deployed artifact, with a deterministic reproducible proof where required.

---

# 2. Executive conclusion

## Final result

**NO CONFIRMED BOUNTY-GRADE FINDING.**

The investigation was not a superficial source review. It examined multiple connected state machines involving V2 offers, BUFFER, redemption, vault liquidity, MarketStats, circulating-supply exclusions, Prop-AMM cadence state, authorization, PDA binding, and deployment correlation.

The strongest non-trivial behavioral candidate was a Prop-AMM cadence-state griefing effect. A deterministic harness demonstrated that repeated dust sells can increase `curr_sell_trade_count`, saturate the cadence mechanism, and worsen a subsequent victim quote. The measured scenario used approximately 19 additional dust sells of 875 ONyc base units each. However, the resulting effect was characterized as liquidity/DoS-style degradation, which is outside the relevant bounty impact class, and the available evidence did not establish cryptographic equivalence to the latest deployed artifact. It was therefore not promoted to a bounty finding.

The redemption lifecycle was separately analyzed as a state-machine/accounting problem. The investigation did not reproduce double settlement, over-return on partial cancellation, request replay, unauthorized request substitution, or an unexplained vault/liability imbalance. A deterministic accounting regression model was added to the personal fork to preserve these invariants.

MarketStats analysis identified freshness and refresh-order semantics, including the cached excluded-balance dependency. Those semantics can produce a stale snapshot until the appropriate update/refresh sequence occurs, but no eligible unauthorized extraction, privilege bypass, or bounty-grade financial impact was demonstrated.

The investigation therefore ends with a disciplined negative conclusion: **no confirmed bounty-grade issue is claimed on the evidence available.**

---

# 3. Investigation discipline

The investigation was intentionally constrained to the authorized OnRe bounty lane and safe local testing.

The working rule was to avoid broad speculative exploration and instead attack the strongest remaining chains using a state-machine approach.

For each hypothesis, the following questions were applied:

1. What can the attacker control?
2. What exact instruction/state transition is required?
3. What account or PDA relationships constrain that transition?
4. What invariant would have to fail?
5. Can the sequence be reproduced deterministically?
6. Does the result create a bounty-eligible impact?
7. Is the affected behavior actually deployed?
8. Can the deployed artifact be correlated to the source/harness being tested?
9. If any required condition fails, can the hypothesis be closed without continuing speculative work?

This prevented several interesting but non-actionable behaviors from being incorrectly promoted to findings.

No unauthorized scanning, credential abuse, persistence, destructive activity, DDoS/spam, social engineering, or scope bypass was used.

---

# 4. Investigation map

The principal chains were:

### Chain A
`permissionless V2 → BUFFER/main-offer accrual → redemption-vault refill → MarketStats`

### Chain B
`redemption create → partial fulfill → cancel remaining → recreate → settlement`

### Chain C
`BUFFER/max-supply boundary → redemption`

### Chain D
`Prop-AMM sell → cadence state → redemption liquidity/accounting`

### Chain E
`cached excluded balance → MarketStats → downstream pricing`

Additional cross-cutting hypotheses included:

- redemption replay/double-use;
- request-PDA substitution;
- redemption-offer substitution;
- mint/state account substitution;
- worker/boss/redeemer authorization confusion;
- canonical `main_offer` versus alternate-offer state confusion;
- stale MarketStats snapshots;
- refresh ordering;
- uninitialized excluded-balance behavior;
- vault refill/target behavior;
- partial fulfillment accounting;
- cancellation accounting;
- multiple-request global reconciliation;
- deployment/source equivalence;
- whether existing fuzz infrastructure already covered the relevant invariant space.

---

# 5. Repository state and versioning

The upstream repository reviewed was:

`onre-finance/onre-sol`

The current master SHA used for the investigation was:

`c0490831f438074562ebf51e437ca5f8e027596b`

A separate deployment correlation previously identified commit:

`1dbc6ec0ed18a8fc81516f930d85e669b689d852`

The distinction between these commits is material. A source-level or local-harness observation cannot automatically be represented as a live-program vulnerability unless the affected code/state can be tied to the deployed artifact.

The investigation therefore treated deployment correlation as a first-class confirmation requirement rather than an administrative afterthought.

---

# 6. Open repository work relevant to the investigation

## PR #149 — Crucible / LiteSVM invariant harness

The repository contains an open PR titled:

`fuzz: crucible SVM invariant harness for onreapp + FuzzCorp CI`

The reviewed PR is additive fuzz/invariant infrastructure. It builds `onreapp` from source, runs Crucible/LiteSVM-style invariant checks, asserts invariants after instructions, and integrates with an external fuzzing workflow.

The PR uses an `FUZZ_API_KEY` for its external workflow and specifies the relevant SBF tools version in its configuration.

This infrastructure was treated as evidence that invariant-oriented testing is appropriate for the program. It was not represented as an exploit proof and was not used to claim that every relevant state transition had already been exhaustively fuzzed.

**Disposition:** supporting infrastructure; no independent security finding.

## PR #153 — Program V6

The repository also contains open PR #153, titled:

`Program V6`

The reviewed changes include:

- removal of V1 offer instructions;
- removal of several legacy MarketInfo getters;
- movement of canonical market metrics toward `MarketStats`;
- MarketStats as a cached snapshot associated with `state.main_offer`;
- freshness metadata including `lastUpdatedAt` and `lastUpdatedSlot`;
- permissionless refresh behavior;
- explicit excluded-balance update/refresh ordering;
- continued presence of V2 redemption-related instructions.

This PR was important because it changes how downstream consumers are expected to reason about market information. It was treated as architecture/state-model evidence, not as a vulnerability claim.

---

# 7. MarketStats architecture reviewed

The current master still contains the MarketInfo modules, including the MarketStats and refresh paths.

The refresh path examined was:

`programs/onreapp/src/instructions/market_info/refresh_market_stats.rs`

The relevant account/state relationships include:

- the canonical `main_offer` associated with the program state;
- token-in mint validation;
- ONyc mint validation;
- the circulating-supply excluded-balance PDA;
- the `MarketStats` PDA;
- permissionless refresh behavior;
- initialization of MarketStats when needed;
- recomputation and storage of the snapshot;
- refresh event emission.

The excluded-balance PDA is cached. If it has not been initialized, the refresh path treats the value as zero. If excluded balances or the configured owner set changes, the intended sequence is to update the excluded-balance cache before refreshing MarketStats.

This is an important semantic distinction:

**refreshing MarketStats is not itself equivalent to recomputing every underlying cache.**

The investigation therefore tested whether that distinction could become an exploitable discrepancy rather than merely a stale-data condition.

---

# 8. Chain A — Permissionless V2 → BUFFER → redemption-vault → MarketStats

## 8.1 Hypothesis

A permissionless V2 action could alter the effective offer/BUFFER state. A subsequent redemption-vault operation might then observe a different state from MarketStats or another downstream consumer.

The potential attack shape was:

`attacker-controlled V2 state change → BUFFER/main-offer state changes → redemption/vault state changes → MarketStats observes stale or inconsistent values → downstream pricing/accounting effect`

## 8.2 Attacker control

The important question was not whether a permissionless instruction exists, but whether its state transition can be composed with another instruction to cross a security boundary.

The investigation therefore followed the complete chain rather than stopping at the fact that a refresh or update was permissionless.

## 8.3 State relationships considered

The analysis considered:

- canonical `state.main_offer`;
- V2 offer state;
- BUFFER behavior;
- redemption-vault target/refill behavior;
- MarketStats snapshot state;
- refresh timing;
- excluded-balance cache state.

## 8.4 Observed behavior

The relevant effect reduced to a state freshness/order dependency. A downstream MarketStats consumer can observe a pre-refresh snapshot until the appropriate refresh occurs.

This is materially different from an attacker obtaining assets, changing a protected configuration, bypassing authorization, or creating an accounting deficit.

## 8.5 Impact test

The investigation attempted to determine whether the lag could be transformed into:

- unauthorized asset extraction;
- incorrect redemption settlement;
- permanent accounting corruption;
- privilege escalation;
- an eligible victim loss;
- or another impact category recognized by the bounty.

No such eligible impact was established.

## 8.6 Result

**ELIMINATED as a confirmed bounty finding.**

The state freshness behavior is recorded because it is relevant to future changes, but it does not meet the confirmation threshold.

---

# 9. Chain B — Redemption lifecycle state machine

This was the deepest accounting-focused investigation.

The target lifecycle was:

`create → partial fulfill → cancel remainder → create another request → fulfill/settle`

The central concern was whether the state machine could create or destroy value when transitions were composed rather than executed in isolation.

---

# 10. Redemption accounting model

Let:

- `C` = cumulative amount created through redemption requests;
- `V` = amount still locked against pending redemption requests;
- `S` = amount already fulfilled/settled;
- `R` = amount returned through cancellation;
- `Q` = remaining redemption liability.

The primary conservation equation is:

`C = V + S + R`

The pending-liability relationship is:

`Q = V`

These equations must hold after each state transition.

A final-state-only check is insufficient because an intermediate transition could temporarily create an inconsistent state that a later instruction accidentally masks.

Therefore the regression model checks conservation after creation, after fulfillment, and after cancellation.

---

# 11. Redemption creation

For a request amount `A`:

`Q := Q + A`

`V := V + A`

No amount should simultaneously appear as settled or returned.

The creation transition therefore establishes the initial liability and locked amount.

A failure at this stage would require either:

- a liability increase without corresponding locked assets;
- locked assets without corresponding liability;
- or some other account relationship that breaks conservation.

No such break was established in the investigated lifecycle.

---

# 12. Partial fulfillment

For fulfillment amount `F`:

`F <= Q`

and:

`F <= V`

The expected transition is:

`Q := Q - F`

`V := V - F`

`S := S + F`

The critical property is that fulfillment reduces the pending liability by exactly the fulfilled amount.

The investigation specifically considered whether partial fulfillment could leave the request in a state where cancellation later returned the already-settled portion.

No reproducible double-return path was established.

---

# 13. Cancellation after partial fulfillment

Suppose:

`A = original request amount`

`F = already fulfilled amount`

Then the remaining cancellable amount must be:

`K = A - F`

The expected transition is:

`Q := Q - K`

`V := V - K`

`R := R + K`

Returning `A` instead of `A - F` would double-account the fulfilled portion.

This was explicitly encoded as a regression condition.

---

# 14. Redemption replay analysis

A separate question was whether a request could be fulfilled or cancelled again after the state transition had already consumed it.

The investigation considered:

- request lifecycle state;
- account closure;
- request PDA identity;
- redemption-offer binding;
- signer constraints;
- partial fulfillment followed by cancellation;
- subsequent creation of another request.

No reproducible sequence survived in which the same economic liability could be settled twice.

**Disposition:** replay/double-use hypothesis eliminated.

---

# 15. Redemption PDA binding analysis

The investigation examined whether an attacker could substitute a different redemption request or redemption offer account into a lifecycle instruction.

The relevant protection model includes PDA/account relationships and explicit matching between the request and its associated redemption offer.

The question was whether an attacker could construct a valid-looking alternate account satisfying enough superficial constraints to redirect settlement.

No reproducible unauthorized substitution path was established.

**Disposition:** eliminated.

---

# 16. Redemption authorization analysis

The investigation separately considered the authority boundaries between:

- the original redeemer;
- the configured worker/boss authority;
- and other arbitrary signers.

The concern was a confused-deputy pattern in which an attacker could cause an authorized actor's authority to be applied to attacker-selected state.

The relevant signer/account constraints were inspected as part of the lifecycle analysis.

No eligible unauthorized financial advantage was established.

**Disposition:** eliminated.

---

# 17. Redemption regression tests added

The personal fork contains:

`programs/onreapp/tests/redemption_settlement_invariants.rs`

The deterministic model implements a minimal ledger with:

- `create`;
- `fulfill`;
- `cancel`;
- conservation checking.

The three scenarios are:

### Test 1
`create_partial_fulfill_cancel_reconciles`

Creates 1,000, fulfills 400, and cancels 600.

Expected final accounting:

- requested: 0;
- locked/vault: 0;
- settled: 400;
- returned: 600;
- total: 1,000.

### Test 2
`multiple_requests_reconcile_independently_and_globally`

Uses two requests of 1,000 and 750 and processes them through a mixture of fulfillment and cancellation.

Expected global accounting:

- requested: 400;
- locked/vault: 400;
- settled: 850;
- returned: 500;
- created: 1,750.

### Test 3
`cancellation_returns_only_the_remaining_request`

Creates 900, fulfills 350, and cancels only 550.

The test explicitly prevents the already-settled 350 from being returned a second time.

These tests are regression coverage, not an exploit PoC.

---

# 18. Chain C — BUFFER/max-supply boundary → redemption

## Hypothesis

A boundary condition involving BUFFER and maximum supply could cause redemption to accept a state that should have been rejected, potentially creating an accounting or supply imbalance.

## Questions examined

- Can a permissionless state transition move the system across the maximum-supply boundary?
- Does redemption validate the resulting state sufficiently?
- Can a boundary value be represented differently between BUFFER and redemption?
- Can the resulting state create more redeemable liability than backed assets?
- Can an attacker turn the boundary into extraction rather than merely rejection/failure?

## Result

No reproducible eligible bypass was established.

No persistent accounting deficit, unauthorized mint-like effect, or bounty-eligible extraction path survived the analysis.

**Disposition: ELIMINATED.**

---

# 19. Chain D — Prop-AMM cadence-state investigation

This was the most interesting behavioral candidate.

## 19.1 Hypothesis

Repeated low-value sells might modify internal Prop-AMM cadence state. If that state affects a later victim quote, a third party might be able to impose a persistent or economically meaningful degradation on another user's transaction.

The conceptual sequence was:

`attacker dust sells → curr_sell_trade_count increases → cadence threshold/haircut changes → victim receives worse quote → possible redemption/liquidity impact`

## 19.2 State variable

The investigated Prop-AMM path increments:

`curr_sell_trade_count`

on successful sells and incorporates the cadence state into the relevant haircut/quote calculation.

## 19.3 Deterministic harness observation

A third-party/local deterministic harness produced a reproducible state-griefing scenario.

The measured sequence was approximately:

- 19 additional dust sells;
- 875 ONyc base units per sell;
- cadence saturation;
- a worsened subsequent victim quote.

This was a meaningful technical observation and was retained rather than silently discarded.

## 19.4 Why it was not reported as a bounty finding

Two independent confirmation conditions failed.

### Impact classification

The effect was characterized as liquidity/DoS-style degradation. The relevant bounty scope excludes lack-of-liquidity-style impact.

### Deployment correlation

The available evidence did not establish cryptographic equivalence between the tested source/state and the latest deployed artifact.

Therefore:

`reproducible behavior != confirmed bounty vulnerability`

The observation remains useful for engineering review and future scope changes, but it is not claimed as a bounty win.

**Disposition: technically observed; bounty finding rejected.**

---

# 20. Prop-AMM follow-up questions considered

The investigation considered whether the cadence effect could be upgraded from a liquidity grievance into a different eligible category.

Questions included:

- Could the cadence state cause permanent asset loss?
- Could an attacker receive assets they did not own?
- Could the victim's loss be transferred directly to the attacker?
- Could the state survive beyond the intended cadence window in a way that changes impact classification?
- Could a redemption settlement become incorrect rather than merely worse-priced?
- Could a protected state transition be bypassed?

No evidence sufficient to establish an eligible category was obtained.

The correct disposition remained conservative: preserve the behavior, but do not overclaim it.

---

# 21. Chain E — Cached excluded balance → MarketStats → downstream pricing

## Hypothesis

A stale excluded-balance cache could make MarketStats compute from an outdated circulating-supply exclusion value and thereby influence downstream pricing or NAV-related behavior.

## Architecture

The refresh path reads the excluded-balance PDA rather than independently recomputing the exclusion set every time.

The intended operational relationship is:

`excluded-balance update → MarketStats refresh → downstream consumer`

If the first step is omitted, the second step can legitimately operate on a stale cached value.

## Questions examined

- What happens when the excluded-balance PDA is uninitialized?
- Can an attacker control its value directly?
- Can an attacker refresh MarketStats without first refreshing the exclusion cache?
- Does the resulting snapshot persist?
- Are `lastUpdatedAt` and `lastUpdatedSlot` sufficient for consumers to recognize staleness?
- Can the stale value be converted into an eligible financial exploit?

## Result

The analysis established a freshness/order dependency but not an unauthorized accounting exploit.

The behavior is particularly important for integrators because a cached snapshot has different semantics from an always-live calculation.

No bounty-grade extraction, privilege escalation, or eligible permanent financial loss was demonstrated.

**Disposition: known freshness semantics / eliminated as bounty finding.**

---

# 22. Uninitialized excluded-balance state

The refresh logic treats an uninitialized excluded-balance PDA as zero.

This was examined because a first-time refresh can therefore differ from a later refresh after the exclusion cache has been populated.

The relevant question was whether an attacker could intentionally exploit the zero default to create a protected financial state that could not otherwise be produced.

No eligible exploit sequence was established.

The behavior is better understood as an initialization/freshness property requiring consumers and operators to respect the documented update ordering.

**Disposition: eliminated.**

---

# 23. MarketStats refresh permissionlessness

MarketStats refresh is permissionless, and the signer can pay for first-time PDA creation.

Permissionlessness alone was not treated as a vulnerability.

The investigation asked whether an arbitrary signer could choose attacker-controlled accounts or state such that the refresh would write attacker-selected market information.

The canonical `main_offer` and mint/state relationships were relevant protections.

No reproducible attacker-controlled substitution of the canonical market state was established.

**Disposition: eliminated.**

---

# 24. Canonical main_offer analysis

The investigation repeatedly returned to `state.main_offer` because several downstream calculations depend on canonical offer state.

The concern was whether alternate offer state or global BUFFER state could be confused with the canonical offer and thereby produce an exploitable mismatch.

The analysis considered:

- which account is accepted as the main offer;
- whether the state relationship is validated;
- token-in mint consistency;
- ONyc mint consistency;
- whether downstream snapshots are derived from the canonical offer.

No reproducible eligible mismatch was established.

**Disposition: eliminated.**

---

# 25. Redemption-vault refill / target analysis

The redemption-vault path was reviewed as part of both Chain A and Chain B.

The question was whether vault target/refill behavior could be manipulated so that:

- redemption liabilities exceed backed assets;
- a refill operation can be triggered against attacker-selected state;
- or a temporary liquidity state becomes permanent accounting corruption.

No bounty-grade accounting deficit or unauthorized extraction was demonstrated.

Temporary liquidity/freshness effects were not promoted to security findings.

---

# 26. Multiple-request accounting

Single-request correctness can hide bugs that only appear when requests interact.

The investigation therefore included multiple independent requests and global reconciliation.

The expected relationship remains:

`total created = total locked + total settled + total returned`

The deterministic regression model verifies this across multiple requests rather than assuming that each request is processed in isolation.

No cross-request accounting leak was reproduced.

---

# 27. Partial fulfillment and cancellation edge cases

The investigation explicitly separated these cases:

1. full fulfillment;
2. partial fulfillment followed by cancellation;
3. cancellation without fulfillment;
4. multiple partial fulfillments;
5. creation of a new request after cancellation;
6. global reconciliation after mixed request states.

The most important accounting rule is that cancellation can only return the still-unfulfilled portion.

No tested state established a double-return or unexplained liability reduction.

---

# 28. Authorization versus accounting

A security issue can arise either from incorrect accounting or from an unauthorized actor being allowed to invoke an otherwise-correct transition.

The investigation therefore kept these questions separate:

### Accounting question
Does the transition conserve value?

### Authorization question
Who is allowed to cause the transition?

### Binding question
Which exact request/offer/state is the transition allowed to affect?

### Deployment question
Is the behavior actually present in the deployed artifact?

A hypothesis had to survive all four dimensions to become a candidate finding.

---

# 29. Evidence hierarchy used

Evidence was treated in the following order:

1. deployed-artifact evidence;
2. deterministic end-to-end reproduction;
3. exact source-level account/state constraints;
4. deterministic local accounting model;
5. static reasoning about hypothetical state;
6. theoretical possibility.

The lower levels were useful for hypothesis generation but insufficient by themselves for a confirmed bounty finding.

This prevented theoretical state discrepancies from being mistaken for deployed exploits.

---

# 30. What was actually reproduced

The investigation produced the following concrete results:

### Reproduced / supported

- deterministic redemption accounting relationships;
- conservation across create/partial-fulfill/cancel in the regression model;
- multiple-request global reconciliation in the regression model;
- the specific partial-cancellation rule;
- Prop-AMM cadence-state degradation in the reported deterministic harness conditions;
- repository-level MarketStats refresh and cached excluded-balance semantics;
- repository-level redemption authorization/PDA relationships relevant to the reviewed lifecycle.

### Not reproduced as a confirmed exploit

- double redemption settlement;
- unauthorized cancellation;
- request-PDA substitution;
- redemption-offer substitution;
- permanent MarketStats corruption;
- eligible financial extraction from stale MarketStats;
- BUFFER/max-supply bypass;
- permanent redemption-vault accounting deficit;
- eligible exploit on the latest cryptographically correlated deployment.

---

# 31. What was deliberately not claimed

This report does **not** claim:

- that a vulnerability was accepted by OnRe;
- that a bounty was awarded;
- that payment was received;
- that the Prop-AMM cadence behavior is bounty-eligible;
- that current master is identical to every deployed artifact;
- that the local accounting model is an end-to-end LiteSVM exploit;
- that open PR #149 or #153 is merged;
- that any upstream PR was successfully submitted through the connected GitHub integration.

These distinctions are intentional.

---

# 32. Negative-result significance

A negative security result is useful only if it preserves enough detail to prevent the same hypotheses from being repeatedly reopened without new evidence.

This report therefore records not just “no issue,” but the exact classes of hypotheses that were tested and why they were closed.

Future investigation should reopen a closed hypothesis only if at least one material fact changes, such as:

- a new deployed artifact;
- changed account constraints;
- changed bounty impact rules;
- changed redemption accounting;
- changed MarketStats semantics;
- changed Prop-AMM cadence implementation;
- a new deterministic reproduction;
- or new evidence tying a previously tested behavior to the live program.

---

# 33. Regression artifact and commits

The focused redemption test is in:

`programs/onreapp/tests/redemption_settlement_invariants.rs`

Original test commit:

`65da4b8055b26b9be806d1c1dcfcf69bd2c7921e`

The focused redemption report is:

`reports/redemption-settlement-invariant-report.md`

The broader investigation report is:

`reports/onre-complete-investigation-report.md`

This expanded report is:

`reports/onre-complete-investigation-report-v2.md`

---

# 34. Personal fork status

The investigation artifacts are preserved in:

`CMKIXVI/onre-sol`

The dedicated expanded-report branch is:

`reports/complete-onre-investigation`

The original redemption work is also preserved in:

`test/redemption-settlement-invariants`

The upstream PR creation attempt was blocked by the GitHub integration permission boundary with a `403 Resource not accessible by integration` response.

Accordingly, this report makes no claim that OnRe upstream has received, reviewed, accepted, or merged the work.

---

# 35. Recommended engineering disposition

Even though no bounty-grade issue was confirmed, several engineering protections are worth preserving.

## Redemption

Keep deterministic accounting invariants covering:

- creation;
- partial fulfillment;
- cancellation;
- multiple requests;
- global conservation.

## MarketStats

Keep explicit freshness semantics and make consumers aware that MarketStats is a snapshot rather than a continuously recomputed value.

## Excluded balance

Keep the update-before-refresh ordering explicit and test the uninitialized-cache case.

## Prop-AMM

Retain the cadence-state behavior as an engineering observation. If impact rules or implementation change, reassess whether the state-griefing behavior becomes security-relevant.

## Deployment correlation

For any future bounty candidate, establish exact deployed-program equivalence before investing in a final submission package.

---

# 36. Suggested future regression matrix

A future comprehensive test suite should include:

| Area | Scenario | Required invariant |
|---|---|---|
| Redemption | create | liability equals locked amount |
| Redemption | partial fulfill | liability decreases exactly by fulfilled amount |
| Redemption | cancel | only remaining amount returns |
| Redemption | partial + cancel | settled amount never returns |
| Redemption | multiple requests | global conservation |
| Redemption | recreate | previous request cannot be reused |
| Redemption | authorization | unauthorized signer rejected |
| Redemption | PDA binding | request cannot be redirected |
| MarketStats | refresh | canonical main_offer used |
| MarketStats | stale cache | freshness metadata remains coherent |
| MarketStats | excluded balance | update/refresh ordering preserved |
| MarketStats | uninitialized cache | zero/default semantics explicit |
| BUFFER | max supply | boundary cannot create excess liability |
| Prop-AMM | dust sells | cadence behavior remains bounded/intended |
| Deployment | artifact | source revision matches deployed program |

---

# 37. Final hypothesis disposition matrix

| ID | Hypothesis | Evidence/result | Final status |
|---|---|---|---|
| A | V2 → BUFFER → redemption → MarketStats | freshness/order behavior only | Eliminated |
| B | redemption partial fulfill → cancel | conservation held | Eliminated |
| B2 | redemption replay | no surviving replay sequence | Eliminated |
| B3 | request/offer substitution | PDA/account binding held | Eliminated |
| B4 | worker/redeemer confusion | no unauthorized advantage | Eliminated |
| C | BUFFER/max-supply → redemption | no eligible bypass | Eliminated |
| D | Prop-AMM cadence griefing | behavior reproduced; impact excluded | Not bounty-eligible |
| E | excluded cache → MarketStats | stale snapshot semantics | Eliminated |
| E2 | uninitialized exclusion cache | no eligible exploit | Eliminated |
| E3 | permissionless MarketStats refresh | no attacker-controlled canonical state | Eliminated |
| F | alternate offer/global BUFFER confusion | no eligible mismatch | Eliminated |
| G | deployment mismatch | correlation not proven | Confirmation blocker |

---

# 38. Confirmation standard

For completeness, the investigation's confirmation standard can be stated as:

`CONFIRMED = reproducible exploit × eligible impact × deployed-artifact correlation`

If any factor is absent, the result remains a hypothesis, engineering observation, regression test, or eliminated path rather than a confirmed bounty finding.

This standard was particularly important for the Prop-AMM result, because that work produced a real reproducible behavior but failed the impact/deployment requirements for bounty confirmation.

---

# 39. Overall technical assessment

The OnRe program contains several stateful components whose security cannot be evaluated solely by checking individual instruction authorization.

The most important security properties identified during this investigation are:

1. **value conservation** across redemption state transitions;
2. **liability-to-locked-asset reconciliation**;
3. **request identity binding**;
4. **authority separation** between redeemer and worker/boss roles;
5. **canonical market-state binding**;
6. **snapshot freshness semantics**;
7. **cached excluded-balance ordering**;
8. **supply/liquidity boundary preservation**;
9. **cadence-state boundedness**;
10. **exact source-to-deployment correlation**.

The investigation did not establish a failure of these properties that simultaneously produced an eligible deployed vulnerability.

---

# 40. Final conclusion

## Security status

**NO CONFIRMED BOUNTY-GRADE VULNERABILITY.**

## Strongest observed behavior

**Prop-AMM cadence-state griefing:** approximately 19 dust sells at approximately 875 ONyc base units each were sufficient in the tested harness conditions to saturate cadence state and worsen a subsequent quote.

## Why it is not a finding

The observed impact was liquidity/DoS-style degradation outside the relevant bounty impact class, and exact current-deployment equivalence was not established.

## Strongest preserved invariant

For redemption accounting:

`created = locked + settled + returned`

with:

`pending liability = locked`

and cancellation after partial fulfillment returning only:

`original amount - already fulfilled amount`.

## Final blocker

No hypothesis simultaneously satisfied:

- reproducible end-to-end exploit;
- eligible bounty impact;
- and current deployed-artifact correlation.

Therefore the correct disposition is to preserve the evidence, keep the regression coverage, and avoid making an unsupported bounty claim.

---

# Appendix A — Compact investigation chronology

1. Refreshed repository and bounty context.
2. Identified the strongest remaining state-transition chains.
3. Reviewed current master and relevant deployment correlation.
4. Examined V2/BUFFER/main-offer relationships.
5. Examined redemption lifecycle and cancellation/fulfillment relationships.
6. Examined BUFFER/max-supply boundary behavior.
7. Investigated Prop-AMM cadence state.
8. Investigated cached excluded balance and MarketStats.
9. Reviewed redemption authorization and PDA binding.
10. Reviewed open PR #149 invariant infrastructure.
11. Reviewed open PR #153 MarketStats/V6 architecture.
12. Recorded the Prop-AMM cadence observation and rejected it as a bounty finding.
13. Built deterministic redemption accounting regression coverage.
14. Preserved the focused report.
15. Consolidated the complete investigation into this expanded report.

---

# Appendix B — Evidence discipline

The investigation deliberately distinguishes:

**Observed:** behavior directly seen in the tested model/source state.

**Reproduced:** behavior repeatable under deterministic conditions.

**Eligible:** behavior whose impact belongs to the bounty's accepted impact classes.

**Deployed:** behavior established to exist in the relevant live artifact.

**Confirmed:** all required dimensions satisfied.

The Prop-AMM cadence observation reached **Observed + Reproduced**, but not **Eligible + Deployed**, and therefore not **Confirmed**.

The redemption invariant work reached **Observed + Reproduced as a model**, but it did not reveal a production accounting failure, and therefore remains **Regression Coverage**, not a vulnerability.

---

# Appendix C — Reopening conditions

The investigation should only be reopened for a previously eliminated hypothesis if new evidence changes the conclusion.

Examples:

- a new program deployment matches the investigated commit;
- bounty impact rules change;
- redemption code changes;
- MarketStats semantics change;
- Prop-AMM cadence logic changes;
- a real end-to-end LiteSVM reproduction demonstrates a new state transition;
- new deployment metadata establishes exact artifact equivalence;
- a previously excluded impact becomes explicitly eligible.

Without such a change, repeating the same hypothesis does not materially increase confidence.

---

# Appendix D — Final statement

This document is intended to be a durable record of the complete investigation, including both positive observations and negative results.

It intentionally avoids converting engineering observations into unsupported vulnerability claims.

**Final status: investigation complete; no confirmed bounty-grade finding.**
