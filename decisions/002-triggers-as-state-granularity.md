# ADR-002: Triggers and Dilemmas as State Granularity

**Date**: 2026-02-25

**Status**: Accepted

## Context

During language design, we considered introducing a `trigger` or `reacts_to` keyword to model how one technique creates a predictable reaction that enables another technique. Common examples in martial arts:

- **Hip bump to triangle (BJJ)**: The hip bump attempt forces the opponent to base with their hands, which exposes the neck for a triangle choke.
- **Collar tie pull to blast double (Wrestling)**: Pulling the collar tie forces the opponent to posture up, exposing their hips for a double leg takedown.
- **Strikes to the head to open body attacks (Boxing)**: Repeated jabs to the head force the opponent to raise their guard, uncovering the body for hooks and uppercuts.

These are **dilemmas** — situations where the opponent must react, and every possible reaction leads to a different attack opportunity for the person creating the situation.

The question was whether the language needs dedicated syntax to express this "if they react with X, I do Y" logic.

## Decision

We decided that **triggers and dilemmas are modeled through state granularity** — by defining more specific sub-states that capture the structural vulnerability — rather than through new syntax.

### Reasoning

**1. The "trigger" is a transition into a vulnerable sub-state.** When you hip bump from closed guard, the opponent doesn't stay in the same structural configuration. They either base with their arms (a new configuration) or posture up (a different new configuration). These are structurally distinct states that enable different transitions:

```
state ClosedGuard roles { Top, Bottom }
state ClosedGuardArmBase roles { Top, Bottom }
state ClosedGuardPosturedUp roles { Top, Bottom }

sequence HipBumpToTriangle:
    HipBumpAttempt: ClosedGuard[Bottom] -> ClosedGuardArmBase[Bottom]
    LockTriangle: ClosedGuardArmBase[Bottom] -> TrianglePosition[Bottom]

sequence HipBumpSweep:
    HipBumpAttempt: ClosedGuard[Bottom] -> ClosedGuardPosturedUp[Bottom]
    CompleteSweep: ClosedGuardPosturedUp[Bottom] -> Mount[Top]
```

**2. Dilemmas appear naturally in the graph topology.** A dilemma is a node where every outgoing edge available to one participant leads to a state that disadvantages them. This is a topological property — tooling can detect it by analyzing the graph, without needing it declared in the syntax.

**3. Reaction modeling leads to agent and strategy modeling.** Explicit "if they do X, respond with Y" syntax implies one participant observing and deciding based on the other's action. This is agent modeling and strategy modeling, both of which are explicit non-goals of the DSL (spec §1.2). The language models structural configurations, not decisions.

**4. Granularity is a modeling choice, not a language constraint.** A beginner's system uses `ClosedGuard` as a single state. An advanced system breaks it into `ClosedGuardPosturedUp`, `ClosedGuardArmBase`, `ClosedGuardBrokenPosture`, etc. This reflects the practitioner's depth of understanding — a white belt sees one state where a black belt sees five. The language supports both levels without needing different syntax.

## Consequences

- **State count grows with system sophistication.** An advanced BJJ system may have 50+ states instead of 14. This is intentional — it reflects real martial knowledge.
- **Dilemmas are implicit, not declared.** A future `mat analyze` command could detect dilemma nodes (states where one role has only disadvantageous exits), but the language itself does not encode this.
- **No new keywords needed.** The existing `state`, `sequence`, and transition syntax is sufficient.
- **A future grouping mechanism** (see ADR-003) would help manage the state explosion by allowing related sub-states to be collapsed in visualization and documentation.
