# Unit derivation: semantics RFC

**Status: RFC — semantics only, no implementation.** This document is stage 1 of
[#134](https://github.com/TheAxiomFoundation/axiom-rules-engine/issues/134): the
contract a `kind: unit` rule must satisfy before any code exists. Nothing in it
changes the release line. Supplied membership remains the default and continues
to work; derivation, when it eventually exists, is opt-in per program and
off by default. The concrete case worked throughout is 7 CFR 273.1 (SNAP
household composition), from the eCFR text as of 2026-07-01.

Related: [#118](https://github.com/TheAxiomFoundation/axiom-rules-engine/issues/118)
(relation absence silently became a $0 denial),
[#83](https://github.com/TheAxiomFoundation/axiom-rules-engine/issues/83)
(unknown identifiers silently become inputs),
[#125](https://github.com/TheAxiomFoundation/axiom-rules-engine/issues/125)
(artifact format identities), and the 2026-07-27 from-scratch design documents,
whose semantics this contract is written to survive.

## 1. The problem

RuleSpec can encode what a household gets, but not what a household is.
Entities and their membership relations arrive as dataset inputs. The law that
constitutes them — 7 CFR 273.1's purchase-and-prepare rules for SNAP, filing
units for tax, benefit units elsewhere — cannot be encoded, so every consumer
computes unit membership outside the engine, in uncited, unversioned code. The
single most contested determination in benefits administration happens outside
the audited, traced machinery.

What exists today is predicate-*filtered* membership: a `derived_relation` is a
filtered view over a source relation's candidate tuples (`docs/rulespec.md`;
`src/engine.rs` relation reads). The source supplies candidates; a judgment
formula decides which remain. What does not exist is *partition*: given N
persons and person-level facts, produce the Household entities and their
membership tuples, where the number of units is not pre-given. There is no rule
kind whose output is a set of new entity instances, and the evaluator assumes a
fixed population.

Issue #118 was, at bottom, a membership-supply failure: composition was data
the caller had to get right rather than law the engine could derive and cite.
This RFC defines the semantics under which the engine could derive it.

## 2. Vocabulary

- **Universe** — the persons among whom units are formed, given by a roster
  relation (for SNAP: the co-residents of a dwelling) plus person-level and
  person-pair facts.
- **Constitution rules** — the rules that produce units: grouping, combination,
  separation, and status provisions, each carrying its citation.
- **Base grouping facts** — the assertions the general definition runs on (for
  SNAP: "customarily purchases food and prepares meals with").
- **Combination (merge) edges** — statutorily forced co-membership regardless
  of base facts (spouses; children under 22 with a parent).
- **Separation (cut)** — a statutorily licensed removal of a person-set from
  its group into its own unit (the elderly-disabled exception).
- **Status tag** — a per-person determination that changes participation, not
  the partition (the § 273.1(b)(7) exclusions).
- **Election** — a fact recording that an optional provision was invoked
  ("may be considered a separate household" requires someone to so elect).
- **Derived unit** — an entity instance the engine materializes, with a
  content-derived identity and a cited membership tuple per person.

## 3. The statutory case: 7 CFR 273.1, decomposed

Every operative paragraph of § 273.1 falls under one of five operation kinds.
This is the evidence that the vocabulary above is sufficient for the section
that motivates the feature.

| Provision | Content | Operation kind |
|---|---|---|
| (a)(1)–(3) | individual alone; separate purchase-and-prepare; group that purchases and prepares together | base grouping over P&P assertions |
| (b)(1)(i)–(iii) | spouses; person under 22 living with parent; child under 18 under parental control of a non-parent member — "must be considered as customarily purchasing … even if they do not do so," "unless otherwise specified" | combination edges, expressly defeasible only where another provision says so (the foster-child carve-out in (b)(1)(iii) is one such defeat) |
| (b)(2) | elderly disabled individual "may be considered, together with his or her spouse … a separate household," "[n]otwithstanding the provisions of paragraph (a)," subject to a 165-percent-of-poverty-line income test on the others | guarded, elective separation whose notwithstanding-clause reaches (a) only |
| (b)(3)–(b)(6) | boarders, foster individuals, roomers, live-in attendants; boarder status turns on payment vs. "the maximum SNAP allotment for the appropriate size of the boarder household"; boarders/foster individuals participate with the providing household "only at the request of" that household; "[p]ersons described in paragraph (b)(1) … must not be considered" roomers or live-in attendants | classifications with elections; parameter thresholds keyed by a *candidate* group's size; explicit (b)(1) precedence |
| (b)(7)(i)–(xii) | ineligible members: aliens/students (§§ 273.4, 273.5), SSN failure (§ 273.6), work-requirement and IPV disqualifications, institution residents with shelter/treatment exceptions, ABAWD time limit (§ 273.24), and others | status tags — the person's participation changes, the partition does not; the institution-resident exceptions in (b)(7)(vi) are separations "subject to the mandatory household combination requirements of paragraph (b)(1)" |
| (c) | unregulated situations: "the State agency may apply its own policy … if the policy is applied fairly, equitably and consistently" | delegated discretion — an explicit hook, not silence |
| (d) | head of household designation | a determination *within* a derived unit; out of scope for derivation (§ 12) |
| (e) | strikers | a unit-scoped eligibility rule that *reads* membership; phase-2 law, not constitution law (§ 12) |

Three textual anchors matter for the algebra and recur below:

1. **(b)(1) survives separations unless expressly defeated.** (b)(5) and
   (b)(6) each state that (b)(1) persons "must not be considered" roomers or
   live-in attendants; (b)(7)(vi)'s institution-resident exceptions are
   separate households "subject to the mandatory household combination
   requirements of paragraph (b)(1)"; and (b)(2)'s notwithstanding clause
   names (a) only. Mandatory combinations outrank cuts by the text itself.
2. **Defeat of a mandatory combination is always express.** (b)(1) opens with
   "unless otherwise specified," and where the regulation means to specify
   otherwise it does so in terms — "(other than a foster child)" in
   (b)(1)(iii), completed by (b)(4)'s boarder treatment of foster individuals.
3. **Thresholds inside constitution rules are keyed by candidate groups, not
   derived units.** The boarder test compares payment against the maximum
   allotment "for the appropriate size of the boarder household" — the
   boarder-plus-spouse-and-children group whose membership is being decided.
   The (b)(2) income test reads the income of "the others with whom the
   elderly disabled individual *resides*" — the co-resident roster, not the
   derived unit. Both bottom out in person-scoped facts and parameter lookups.
   Neither requires a unit-scoped rule output. § 10 makes this a compile-time
   stratification rule.

## 4. The derivation algebra

The construct is **not** a transitive closure. It is a fixed pipeline of five
operation kinds whose precedence is *declared by citation*, composed as
follows.

### 4.1 Inputs

- A roster relation over persons (for SNAP: co-residence in a dwelling),
  with an explicit completeness assertion (§ 5.1).
- Person facts (age, disability, income items), person-pair facts (spousal
  status, parentage, parental control, purchase-and-prepare assertions), and
  election facts, each with provenance.
- Parameters (poverty guidelines, maximum allotments by size), temporally
  selected as usual.

### 4.2 Pipeline

1. **Edge assembly.** Build a cited edge set over the roster:
   - *base edges* from purchase-and-prepare assertions ((a)(2)–(3)), symmetric;
   - *combination edges* from (b)(1)-class provisions. A combination edge
     exists whenever its trigger facts hold, regardless of base facts, minus
     edges expressly defeated by another provision (the foster carve-out).
     Combination edges are marked indefeasible-by-cuts.
2. **Classification and separation.** Evaluate guards for each
   classification/separation provision — using only person-scoped facts,
   person-scoped derived rules, parameters, and elections (§ 10) — yielding
   zero or more separation sets S₁…Sₖ, each listing its members (e.g., the
   elderly disabled individual and their spouse if living there) and its
   citation.
3. **Blocking.** A separation Sᵢ is *blocked* if any surviving combination
   edge crosses its boundary (one endpoint in Sᵢ, one outside) and the
   separation's provision does not expressly defeat that edge's provision.
   Blocked separations do not apply; the block is a cited determination in the
   trace (§ 6). This is anchor 1 made mechanical.
4. **Partition.** The derived units are:
   - each unblocked Sᵢ as its own unit; and
   - the connected components of the edge graph induced on the remaining
     persons (base ∪ combination edges).
   Singletons fall out naturally as one-person components ((a)(1)–(2)).
   Note removal can split a component: if the separated person was the only
   P&P link between two others, those others become separate households —
   which is what (a)(2) says about people who do not purchase and prepare
   together.
5. **Status tagging.** Apply (b)(7)-class exclusions per person, producing a
   role on each membership tuple: `eligible_member` or
   `excluded_member(citation)`. The partition is unchanged; what downstream
   rules count changes. (Downstream treatment of excluded members' income —
   § 273.11(c) — is ordinary phase-2 law reading the role.)
6. **Emission.** Materialize each unit as an entity instance with a
   content-derived identity (§ 4.4), and one membership tuple per (unit,
   person) carrying role and the citation set that put the person there.
   Tuples are emitted under the relation's single canonical id (§ 11).

### 4.3 Declared precedence, not positional precedence

Every combination, separation, and classification operation declares which
provisions it overrides, quoting the statutory hook ("notwithstanding
paragraph (a)"; "unless otherwise specified"; "other than a foster child").
The compiler builds the defeat graph from those declarations. Two consequences:

- **Undeclared conflict is a compile error.** If two operations can disagree
  about a person's unit and neither declares precedence over the other, the
  program does not compile. There is no document-order, no last-wins, no
  tie-break — consistent with the engine's ambiguity-is-an-error direction
  (#79, #82, #132).
- **The (b)(1)/(b)(2) interaction is decided by the declaration, in the
  open.** On the text, (b)(2) defeats (a) only; a combination edge to a
  person outside {individual, spouse} — say the elderly disabled person's
  21-year-old child in the home — blocks the separation (step 3). Encoding
  that reading requires writing `overrides: [273.1(a)]` and *not*
  `273.1(b)(1)`, and the blocked cut appears in the trace citing (b)(1)(ii).
  If legal review during the pilot concludes FNS practice reads it otherwise,
  the fix is a one-line change to a declaration — visible in review — not a
  buried branch. The algebra's job is to make this question explicit and
  citable, not to answer it.

### 4.4 Determinism

Same facts → same partition, exactly:

- **Order invariance.** Derivation consumes fact *sets*. Permuting dataset
  records cannot change the result (cf. the normalize-once rule in the 2026
  design documents). Duplicate identical assertions deduplicate; contradictory
  assertions about the same pair are a Conflict, handled under § 5.
- **Confluence of separations.** Separation guards are partition-independent
  by construction (§ 10 forbids them reading unit-scoped values), and
  application is set subtraction followed by re-deriving components, so
  separations commute. No application order is observable.
- **Content-derived unit identity.** A derived unit's id is a stable digest of
  (program, unit relation id, sorted member public ids, period). Two runs, or
  two engines, produce the same ids. Insertion order does not exist. Identity
  across *different* periods is out of scope (§ 12).
- **Idempotence.** Re-deriving over the same facts yields the same units and
  tuples, byte-for-byte in canonical serialization.

## 5. Unknown semantics: which missing facts poison which units

The rule #118 established for relations applies with more force here: absence
must never silently become a smaller household or a denial. This section
defines exactly what becomes indeterminate, and — as important — what does not.

### 5.1 Roster completeness is a precondition

Deriving "who is in this household" requires knowing who is in the dwelling. A
derivation consumes its roster relation only under an explicit completeness
assertion ("every person residing at this dwelling in the period appears
here"). Without it, every unit in that roster is indeterminate with reason
`roster_not_asserted_complete` — because an unlisted co-resident could, under
(b)(1), be a mandatory member of any unit.

This is not a new burden invented by derivation; it is the burden the
supplied-membership world was carrying invisibly. A caller who supplies
`member_of_household` tuples is asserting completeness of the very conclusion.
Derivation moves the assertion one level down, onto observable facts, where it
belongs.

### 5.2 The invariance criterion

Let the Unknown facts touching a roster be U. Each u ∈ U has a set of
admissible resolutions (a missing P&P assertion: present or absent; a missing
spousal status: spouse or not; a missing age: any value in its domain, though
guards usually only care about a threshold).

> A derived unit is **Determined** iff its member set and each member's role
> are identical under every admissible joint resolution of U. Otherwise the
> affected persons' unit determinations are **Indeterminate**, and the result
> names the specific members of U that could still change the answer.

Never a guessed partition; never a silently smaller household. The reported
unresolved-fact set is the elicitation product: "which question, if answered,
would determine this household" falls directly out of the criterion.

Two refinements:

- **Partition-determined but role-indeterminate.** If membership is invariant
  but a status guard is Unknown, the unit is Determined as a partition while
  the person's role — and everything downstream that counts eligible members —
  is Indeterminate with the status provision's citation. This is the exact
  shape of the #118 case done right: "cannot determine: SSN cooperation fact
  (§ 273.6, tagged by § 273.1(b)(7)(iv)) not supplied for member p4," rather
  than a confident $0.
- **Elections are not Unknown.** For "may"-provisions, the statutory default
  is the general rule: an unexercised (b)(2) election means the (a)/(b)(1)
  composition stands. An absent election fact therefore resolves to
  not-exercised *by an explicit, trace-recorded default* citing the "may"
  language — it does not poison the unit. (Decision 2 in § 13; the
  strict alternative is stated there.)

### 5.3 Locality: the blast radius is bounded

An Unknown fact poisons at most the units reachable from it:

> Under the coarsest resolution (all Unknown edges present, all Unknown guard
> facts resolved most-connectively), take the connected component(s) touching
> the Unknown fact. Persons outside those components are unaffected: their
> units are Determined if their own facts are complete.

A missing purchase-and-prepare answer between two roommates cannot poison the
elderly couple across the hall of the same dwelling — unless a resolution of
it could actually reach them through some chain of edges or a guard.
Determinations are reported per unit, so determined units of a roster proceed
while indeterminate ones wait for facts. Partial output with per-unit
indeterminacy, never all-or-nothing per roster and never a guess.

Implementation note (non-normative): for merge-only Unknowns, component
coarsening is monotone in the edge set, so evaluating the two extreme
resolutions (all-absent, all-present) suffices: a person whose component is
identical in both is Determined. Guards and cuts break monotonicity, so the
general criterion is quantified over resolutions; the monotone shortcut is an
optimization where it is valid, not the semantics.

### 5.4 Conflicts

Contradictory assertions (two datasets disagree on spousal status; a P&P
assertion and its negation) follow the engine-wide rule: zero covering records
→ Unknown, one → Known, several distinct → Conflict, never resolved by dataset
order. A Conflict poisons exactly as an Unknown does, but is reported as a
conflict with its candidates, since the remedy (adjudicate) differs from the
remedy for absence (ask).

## 6. Cited membership: the trace

"Household size = 3" must be an auditable conclusion. The derivation trace
contains, per unit:

- the roster and its completeness assertion (evidence-carrying);
- every edge that contributed, with its provision and the facts behind it;
- every separation applied — guard evaluation (including the parameter cells
  selected), election evidence, and citation;
- every separation *blocked*, with the blocking combination edge's citation —
  overrides that fired and overrides that were denied are both conclusions;
- per member: the provision(s) that placed them (base edge, combination edge,
  separation carry-along) and their role with its citation;
- for Indeterminate units: the unresolved facts, each tied to the provision
  whose answer it would change.

### Worked example

Dwelling roster (asserted complete): A (age 62, permanently disabled, unable
to purchase and prepare), B (A's spouse), C (A and B's 21-year-old child),
D (roommate), E (D's 8-year-old child). P&P assertions: {A,B,C} together;
D separately with E. Income of C, D, E (the "others" if A elects) is below
165 % of the applicable poverty guideline. A elects (b)(2) separate status.

- Edges: base A–B, A–C, B–C (P&P); D–E base plus combination edge
  ((b)(1)(ii): E, under 22, lives with parent D); combination A–B
  ((b)(1)(i), spouses), A–C and B–C ((b)(1)(ii), under-22 child with
  parents).
- Separation S₁ = {A, B} per (b)(2): guard holds (age, disability, income
  test over co-residents C, D, E using person-scoped income and the
  poverty-guideline parameter), election present.
- **Blocking:** combination edges A–C ((b)(1)(ii)) and B–C cross the S₁
  boundary; (b)(2) declares override of (a) only → **S₁ blocked**, cited to
  (b)(1)(ii). On this text-faithful reading A, B, C remain one household.
  (Had C been 23, no combination edge exists, S₁ applies, and C — who
  purchased and prepared with A and B — becomes a one-person household under
  (a)(2) once A and B are removed.)
- Result: units {A, B, C} and {D, E}; every membership tuple cites its
  provisions, e.g. E ∈ {D,E}: base (a)(3) *and* (b)(1)(ii); the blocked S₁
  appears in {A,B,C}'s trace with both citations.

Sketch of one membership tuple as traced (shape illustrative, format decided
with #125's bump — § 7):

```yaml
unit: household:sha256:9f31…            # content-derived id, § 4.4
person: person:E
role: eligible_member
established_by:
  - provision: us:regulations/7-cfr/273/1#a-3
    via: base_edge {D, E}               # P&P assertion, evidence ref
  - provision: us:regulations/7-cfr/273/1#b-1-ii
    via: combination_edge {D, E}        # age fact + parentage fact refs
derivation: trace_node …                # the unit's full derivation subtree
```

Now delete the P&P fact between D and E (Unknown, not asserted-absent): the
combination edge (b)(1)(ii) alone still connects them — the unit is Determined
with the combination citation only. That is requirement 1 of #134 in one
example: mandatory inclusion overrides missing base facts, citedly. Delete
instead E's parentage fact: the base edge alone connects them — still
Determined, cited to (a)(3). Delete both: {D} vs {D,E} now differ across
resolutions → D's and E's units are Indeterminate, reasons name both missing
facts; A, B, C's unit is untouched (§ 5.3).

## 7. Artifact provenance: derived vs supplied

A consumer must be able to tell whether composition came from law or from the
caller. Requirements on the compiled artifact and the result:

- **Input contract.** Each relation the program uses is marked
  `provenance: supplied` or `provenance: derived {rule, citations}`. A derived
  relation leaves the artifact's required-inputs contract and is replaced by
  the facts its derivation consumes (roster, P&P assertions, ages, elections,
  …) — the artifact's question to the caller changes from "who is in the
  household?" to the facts of § 273.1. `axiom inspect` surfaces this.
- **Result marking.** Every emitted unit and membership tuple is marked
  derived, with the constitution rule's id and the trace root that produced
  it. Supplied tuples pass through marked supplied, as today.
- **Format.** This is an artifact-format change and rides the same format
  bump as #125 — it must not overload format version 2 again. Under #125's
  separation of identities, derivation changes the *semantic* version (a new
  rule kind with defined evaluation) and the *wire* format (new contract and
  provenance fields) together.
- **Prototype constraint.** Stage-2 prototyping (behind the off-by-default
  flag) must not leak: a flag-built artifact carries an explicit
  experimental marker, is rejected by publication tooling, and is never
  emitted without the flag. No change to what v2 consumers see.

Supplied-vs-derived conflict — a program whose artifact derives a relation
receiving that same relation as dataset input — is a bind error by default,
with an explicit comparison mode that accepts both, evaluates from the derived
tuples, and reports discrepancies (the stage-2 harness). Decision 3 in § 13.

## 8. Interaction with #118 and existing machinery

- Derived tuples are emitted under the relation's **single canonical id**, and
  aliases resolve to it. The us-co-snap defect class — several ids for
  `member_of_household`, rules reading different ones, supplied records
  landing on an unread alias — cannot arise for derived membership, because
  emission and reading share one compile-resolved id.
- `derived_relation` (predicate-filtered views) is unchanged and composes
  downstream: a filtered view over a *derived* membership relation is
  well-defined, since filtering runs after materialization.
- Unit entities and tuples enter the same checked-namespace regime as
  everything else (`insert_unique`, #132): a derived unit id colliding with a
  supplied entity id is an error, not a merge.

## 9. Authoring surface (illustrative only)

Non-normative sketch, to make the semantics concrete; the surface is settled
at stage 2:

```yaml
concepts:
  snap_household:
    kind: unit                     # name negotiable per #134
    unit:
      universe:
        roster: dwelling_resident          # requires completeness assertion
      base:
        edges: purchases_and_prepares_with # (a)(2)-(3), symmetric
      combine:
        - id: spouses                      # (b)(1)(i)
          when: spouse_of
          overrides: []
        - id: child_under_22_with_parent   # (b)(1)(ii)
          when: parent_of and age < 22
          overrides: []
      separate:
        - id: elderly_disabled             # (b)(2)
          members: [self, spouse_of]
          guard: age >= 60 and pnp_disabled
                 and others_income <= 1.65 * poverty_guideline(...)
          election: elects_separate_household
          overrides: ["us:regulations/7-cfr/273/1#a"]
      status:
        - id: ssn_noncooperation           # (b)(7)(iv) → § 273.6
          role: excluded_member
          when: ssn_disqualified
      emits:
        entity: household
        relation: member_of_household      # canonical id; aliases resolve here
```

Every `when`/`guard` formula is an ordinary cited judgment; every block
carries proof atoms like any other rule. The two open textual questions found
while working the example — the size key for the (b)(2) poverty-guideline
lookup, which the regulation leaves implicit, and the (b)(1)/(b)(2)
interaction of § 4.3 — are resolved against FNS guidance during the stage-2
pilot, as encoding decisions with citations, not as engine behavior.

## 10. Two-phase evaluation

Constitution rules run before unit-scoped rules. Precisely:

- **Stratification by scope.** Stratum 1: person-scoped and person-pair-scoped
  rules and facts, plus parameters. Stratum U: constitution rules, which may
  read stratum 1 only. Stratum 2: unit-scoped rules, which may read anything.
  The compile-time check: **no dependency path from any unit-scoped rule into
  a constitution rule.** A separation guard referencing `household_income` is
  a compile error naming the offending path.
- This check is what licenses § 4.4's confluence claim, and § 3's anchor 3
  shows 7 CFR 273.1 fits it: the boarder threshold and the (b)(2) income test
  read person facts and parameters keyed by *candidate* group attributes —
  apparent circularity that stratification resolves, since the maximum
  allotment by size is a parameter table, not a phase-2 rule output. A future
  statute that genuinely conditions composition on a derived-unit quantity is
  a simultaneity problem and out of scope here; it would need the declared
  fixpoint escape contemplated by the 2026 re-review, not a silent widening of
  this contract.
- **Materialization barrier.** Between phases the engine materializes derived
  units and tuples; phase 2 evaluates over the enlarged universe exactly as it
  evaluates over a supplied one. Derivation events are ordinary trace nodes;
  phase-2 unit-scoped values point to the membership tuples they aggregated,
  whose traces reach back through the derivation (§ 6).
- **Forward compatibility.** In the from-scratch designs, this is a stratified
  schedule: plan stratum 1 columns, run constitution rules, materialize new
  entity rows and relation indexes, then plan and run stratum 2 — two plans
  and a barrier, not a new evaluator. The current engine's fixed-population
  assumption is the real engineering cost #134 names; this contract is
  deliberately implementable either there (behind the flag) or in a rebuild,
  because it constrains semantics, not machinery.
- **Supplied-membership programs are unaffected.** A program with no
  constitution rules has an empty stratum U; evaluation is exactly today's.

## 11. What derivation is not

- **Not a transitive closure.** The base grouping uses connected components,
  but the construct is closure + declared-precedence merges + guarded cuts +
  status tags. Reducing it to any single closure loses the override algebra
  that is most of the law.
- **Not a guesser.** No canonicalization choice may manufacture a
  determination from incomplete facts. Where the statute is genuinely silent —
  (c)'s unregulated situations, e.g. non-mutual P&P chains under a strict
  reading — the answer is a discretion hook: a state-policy module encodes
  the state's (c) policy with its own citation, or absent one, the unit is
  Indeterminate with reason `discretion_not_encoded(273.1(c))`. Discretion is
  neither Unknown nor a hard-coded interpretation.
- **Not entity resolution.** Derivation partitions an asserted roster of
  distinct persons. Whether two records are the same person is upstream data
  work, out of scope.

## 12. Non-goals

- **Cross-period unit continuity.** Unit identity is per-period (§ 4.4);
  "the same household over time" is real (certification periods, reporting)
  and explicitly deferred.
- **Head of household ((d))** — a designation within a unit, requiring
  election facts and agency-designation fallbacks; representable later as an
  ordinary unit-scoped rule plus elections, and not needed to derive
  composition.
- **Strikers ((e))** — unit-scoped eligibility law reading membership;
  phase 2 as-is. (Its "assuming the strike did not occur" clause is the
  hypothetical-evaluation gap tracked by the re-review, orthogonal to units.)
- **Full (b)(3)–(b)(6) pilot coverage.** The algebra covers boarders, foster
  individuals, roomers, and attendants (classifications + elections +
  candidate-size thresholds); the stage-2 pilot may stage them after the
  (a)/(b)(1)/(b)(2)/(b)(7) core. Semantics complete now; encoding order is a
  pilot decision.
- **No release-line change.** Nothing here blocks or alters current consumers;
  supplied membership remains the default indefinitely.

## 13. The contestable decisions

Everything above follows from the text or the engine's established doctrine
except four genuine choices, listed for ratification:

1. **Non-mutual chains: canonical components, or (c) discretion?** A asserts
   P&P with B, B with C, no assertion between A and C. Components yields
   {A,B,C}; a strict reading calls this a (c) unregulated situation
   (Indeterminate absent a state policy module). *Proposed:* components as
   the canonical rule, with the trace flagging chain-closure inclusions as
   canonicalization-supported (citing (a)(3) and (c)) so they are visible in
   audit; the strict alternative stays available per program.
2. **Unexercised elections: statutory default, or Unknown?** *Proposed:*
   absent election facts resolve to not-exercised via an explicit,
   trace-recorded default citing the "may" language (§ 5.2). Consequence: the
   engine never blocks on options nobody claimed, but a claimant-favorable
   election can go silently unexercised — faithful to administration, worth
   deciding with eyes open. Strict alternative: elections are ordinary facts;
   absence poisons the affected unit.
3. **Supplied ∧ derived: error, or comparison?** *Proposed:* bind error by
   default; explicit comparison mode (derived governs, discrepancies
   reported) for migration and the stage-2 harness. Alternatives: supplied
   silently wins (derivation as mere check) or derived silently wins — both
   reject the engine's no-silent-resolution doctrine.
4. **Excluded members: role-tagged, or dropped?** (b)(7) persons are "not a
   member of any household" textually, yet § 273.11(c) counts part of their
   income to the remaining members — so the unit must still know about them.
   *Proposed:* membership tuples carry `eligible_member` /
   `excluded_member(citation)`, and the pilot maps the existing
   `member_of_household` relation to eligible members so derived output is
   comparable with currently-supplied fixtures. This changes what a bare
   membership count means; downstream rules that should count everyone must
   say so.

Ratifying these four fixes the contract; stage 2 of #134 (off-by-default
prototype, SNAP household pilot in rulespec-us, comparison against supplied
membership on existing fixtures) can then begin against it.
