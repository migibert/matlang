# ADR-001: Generic Actions Over Typed Action Keywords

**Date**: 2026-02-25

**Status**: Accepted

## Context

During language design, we considered replacing the generic action name in sequence steps with typed keywords such as `strike`, `submission`, `transition`, `takedown`, `sweep`, `reversal`, and `guard pass`. The motivation was to make the DSL more intuitive for martial artists by surfacing technique categories directly in the syntax.

For example, instead of:

```
sequence JabCrossClinch:
    Jab: StrikingRange[Orthodox] -> StrikingRange[Orthodox]
    Cross: StrikingRange[Orthodox] -> StrikingRange[Orthodox]
    ClinchEntry: StrikingRange[Orthodox] -> ThaiClinch[Dominant]
```

The proposal was:

```
sequence JabCrossClinch:
    strike Jab (head): StrikingRange[Orthodox] -> StrikingRange[Orthodox]
    strike Cross (head): StrikingRange[Orthodox] -> StrikingRange[Orthodox]
    transition ClinchEntry: StrikingRange[Orthodox] -> ThaiClinch[Dominant]
```

We also considered requiring anatomy targets for strikes and submissions (e.g., `head`, `arm`, `liver`).

## Decision

We decided to **keep actions as generic named steps** without typed keywords or anatomy targets.

### Reasons

**1. Category explosion.** Introducing `strike` and `submission` as keywords immediately forces the introduction of `takedown`, `sweep`, `reversal`, `guard pass`, `escape`, `throw`, `block`, `grip`, `feint`, and more. There is no principled boundary — every discipline introduces its own technique taxonomy. This directly violates the language's design goal of being **discipline-agnostic**.

**2. Categories are non-exclusive.** A single technique often belongs to multiple categories depending on context and intent:

- A **kimura** from closed guard can be a submission, a sweep (to mount), or a reversal (to top position), depending on the opponent's reaction.
- An **uchi mata** in judo can be a throw or a sweep, depending on execution.
- A **guillotine** can be a submission or a takedown defense.

Assigning a single keyword to a technique loses information rather than adding it. The alternative — allowing multiple keywords — adds complexity without structural benefit.

**3. The state transition already encodes technique type implicitly.** The graph topology carries the semantics that keywords would attempt to duplicate:

- `Standing[Neutral] -> ClosedGuard[Top]` is structurally a takedown.
- `Mount[Bottom] -> HalfGuard[Bottom]` is structurally an escape.
- `ClosedGuard[Bottom] -> Mount[Bottom]` is structurally a sweep (positional improvement with role inversion).
- `StrikingRange[Orthodox] -> StrikingRange[Orthodox]` is structurally a strike (self-transition within striking range).
- `ArmbarPosition[Top] -> ArmbarPosition[Top]` is structurally a submission (terminal self-loop in a submission position).

A martial artist reading these transitions already understands the technique type from context.

**4. Anatomy targets are redundant for the target audience.** The expected users of this DSL are martial artists. They already know that an armbar targets the elbow, a jab targets the head, and a heel hook targets the knee. Encoding this in the language adds verbosity without providing structural information that the graph model can use.

**5. Mathematical coherence.** The formal model `M = (S, R, V, Q)` treats all actions uniformly as edges in a directed graph. Introducing typed keywords would require extending the formal model with a technique taxonomy `T` and a classification function `type: Action -> T`, adding complexity to the model without improving its analytical power (reachability, connectivity, and cycle detection are all type-agnostic).

## Consequences

- **Actions remain free-form identifiers.** The action name (e.g., `Jab`, `ArmbarSetup`, `BridgeAndShrimp`) carries the human-readable semantics; the state-role transition carries the structural semantics.
- **The language stays discipline-agnostic.** No built-in knowledge of technique taxonomies means the same grammar works for BJJ, Judo, Boxing, Karate kata, Taekwondo poomsae, and any other two-participant martial art.
- **Future enrichment is possible without breaking changes.** If categorization is needed later, optional **tags or annotations** (e.g., `#strike`, `#submission`) could be added as metadata that doesn't affect validation or the graph — this is noted in the spec as a potential future extension (Section 8: "Action categories").
- **The graph model remains simple and powerful.** Technique type can always be inferred from graph topology by tooling built on top of the DSL, rather than being baked into the grammar.
