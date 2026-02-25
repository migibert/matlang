# ADR-003: State Grouping Over Inheritance

**Date**: 2026-02-25

**Status**: Accepted

## Context

As systems grow in complexity (see ADR-002), the number of states increases to capture sub-configurations such as `ClosedGuardPosturedUp`, `ClosedGuardArmBase`, and `ClosedGuardBrokenPosture`. This raised the question of whether the language should support **inheritance** — e.g., `WilliamsGuard` inheriting from `ClosedGuard` — to express the relationship between parent states and their specializations.

Two interpretations of inheritance were considered:

- **Role inheritance**: A child state automatically receives the parent's valid roles. This saves one line per child declaration — too thin a feature to justify the conceptual weight.
- **Transition inheritance** (Harel statecharts): A child state inherits all transitions defined for the parent. Any sequence step that applies to `ClosedGuard` would also apply to `WilliamsGuard`.

## Decision

We decided **against state inheritance** and in favor of a future **lightweight grouping mechanism** for organizing related states.

### Reasons against inheritance

**1. Transition inheritance is structurally incorrect.** In martial arts, specialized positions typically *do not* share the same available transitions as their parent:

- Williams guard is entered from closed guard, but you can't hip bump from Williams guard (your arm is committed).
- Rubber guard shares the closed guard base, but standard armbar entries don't work (your legs are configured differently).
- Deep half guard shares the half guard entanglement, but has an entirely different sweep and escape set.

If `WilliamsGuard` inherits from `ClosedGuard`, and a hip bump sweep is defined from `ClosedGuard`, the model would incorrectly claim hip bump is available from Williams guard.

**2. The Liskov substitution problem.** Inheritance in a state machine means behavioral substitutability — anywhere the parent state appears in a transition, the child should be valid too. This is almost never true for martial arts positions. The "is-a" relationship a martial artist perceives ("Williams guard *is a type of* closed guard") is about physical resemblance and conceptual lineage, not about shared transitions.

**3. The structural relationship is already expressible.** The connection between a parent state and its specialization is a transition:

```
EnterWilliamsGuard: ClosedGuard[Bottom] -> WilliamsGuard[Bottom]
AbandonWilliams: WilliamsGuard[Bottom] -> ClosedGuard[Bottom]
```

This says: Williams guard is reachable from closed guard, and you can fall back. That's the actual structural relationship — **adjacency**, not inheritance.

### The real need: grouping

The underlying intent was organization and visualization — saying "these 5 states are all closed guard variations" so that:

1. A graph viewer can collapse them into one node for overview
2. Documentation groups them logically
3. There's a named cluster for pedagogical structure

This is a grouping/clustering concern, not an inheritance concern. A future construct could look like:

```
group ClosedGuardFamily {
    ClosedGuard, WilliamsGuard, RubberGuard, ClosedGuardPosturedUp
}
```

### Applicability across disciplines

Grouping is discipline-agnostic and applicable everywhere:

- **Karate/TKD**: Stance families (front stance variations, back stance variations)
- **Judo**: Kuzushi directions (forward, backward, corner — each enabling different throws)
- **Wrestling**: Tie-up variations (collar tie, Russian tie, two-on-one, underhooks)
- **Boxing/Muay Thai**: Range sub-states (long range against orthodox, against southpaw, against fence)
- **Aikido/Jujitsu**: Attack entry families (single wrist grab, double wrist grab, lapel grab)
- **Curriculum design**: Any discipline benefits from collapsible detail levels — a white belt sees one node where an advanced practitioner sees a cluster

## Consequences

- **No inheritance mechanism in V1.** States remain flat — each state has its own explicit role list and transition set.
- **Grouping is deferred to a future version.** It requires no changes to the formal model `M = (S, R, V, Q)` — groups would be metadata for tooling and visualization, not structural primitives.
- **Naming conventions can serve as an interim solution.** States named `ClosedGuard`, `ClosedGuardPosturedUp`, `ClosedGuardArmBase` can be auto-clustered by tooling that recognizes the common prefix.
- **The formal model stays simple.** No type hierarchy, no substitution rules, no multiple inheritance problems.
