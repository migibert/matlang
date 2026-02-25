# Martial Modeling DSL

## Design Document – Version 1.0

---

# 1. Project Vision

## 1.1 Purpose

The goal of this project is to design a **declarative Domain Specific Language (DSL)** for formally modeling the structural logic of a two-participant martial art.

The DSL must:

* Be discipline-agnostic (BJJ, Judo, Boxing, etc.)
* Be mathematically coherent
* Be structurally deterministic at the transition level
* Remain readable and pedagogical
* Avoid modeling damage, scoring, or knockouts
* Avoid modeling agents explicitly

This DSL models **structural positional systems**, not combat simulations.

---

## 1.2 Non-Goals

Version 1.0 explicitly does NOT aim to:

* Simulate fights
* Act as a game engine
* Handle probabilities
* Model biomechanics
* Support more than two participants
* Model time or physics

The system is purely structural and discrete.

---

# 2. Formal Model

A martial system is defined as:

```
M = (S, R, V, Q)
```

Where:

* **S** = finite set of states
* **R** = finite set of roles
* **V ⊆ S × R** = valid state-role combinations
* **Q** = finite set of sequences (ordered action progressions)

---

## 2.1 States (S)

States are atomic structural configurations universally recognized within a discipline.

Examples:

* ClosedGuard
* Mount
* Standing
* Turtle

A state represents a structural configuration of the two-participant system.

States are:

* Atomic (no nesting in V1)
* Finite
* Unique by identifier

---

## 2.2 Roles (R)

Roles represent structural slots, not agents.

They do NOT encode:

* Attacker
* Defender
* Initiative

They encode structural positioning.

Examples:

* Top / Bottom
* Neutral
* Back
* Orthodox / Southpaw

Roles are:

* Finite
* Declared at the system level
* Required for every state reference

---

## 2.3 Valid State-Role Combinations (V)

Not all roles are valid in all states.

For each state `s ∈ S`, we define:

```
valid_roles(s) ⊆ R
```

If a state does not explicitly declare compatible roles:

```
valid_roles(s) = R
```

Then:

```
V = { (s, r) | r ∈ valid_roles(s) }
```

This ensures structural consistency.

Example:

* Standing may only allow Neutral
* Mount may allow Top and Bottom

---

## 2.4 Sequences (Q)

Sequences represent ordered progressions of actions for pedagogical purposes.

A sequence is defined as:

```
Q = { (name, [(action_name, v_from, v_to), ...]) | v_from, v_to ∈ V }
```

Each sequence:

* Has a unique name
* Contains an ordered list of steps
* Each step defines: action name, source configuration, destination configuration
* Steps must form a connected chain: end state of step N = start state of step N+1

Examples:

* BJJ guard passing sequence
* Muay Thai striking combination
* Judo throw setup to pin transition

Sequences are:

* Pedagogical constructs
* Validated for connectivity
* The primary mechanism for defining state transitions

---

# 3. Language Specification (Grammar)

## 3.1 EBNF Grammar

```
program          ::= declaration+

declaration      ::= roles_decl
                   | state_decl
                   | sequence_decl
                   | group_decl

roles_decl       ::= "roles" "{"
                      IDENTIFIER { "," IDENTIFIER }
                     "}"

state_decl       ::= "state" IDENTIFIER [ state_roles ]

state_roles      ::= "roles" "{"
                      IDENTIFIER { "," IDENTIFIER }
                     "}"

sequence_decl    ::= "sequence" IDENTIFIER ":"
                      sequence_step+

sequence_step    ::= IDENTIFIER ":" state_ref "->" state_ref

state_ref        ::= IDENTIFIER "[" IDENTIFIER "]"

group_decl       ::= "group" IDENTIFIER "{"
                      IDENTIFIER { "," IDENTIFIER }
                     "}"
```

**Multi-file Support:**

* A martial system consists of all `.martial` files in a directory
* System name is derived from the directory name
* Role declarations can appear in any file and are merged
* All other declarations are collected across files

---

# 4. Semantic Rules

## 4.1 Uniqueness Constraints

Within a system:

* State identifiers must be unique
* Role identifiers must be unique
* Sequence identifiers must be unique
* Group identifiers must be unique
* Action names within a sequence must be unique

---

## 4.2 Reference Validity

For each state reference:

```
StateName[RoleName]
```

The following must hold:

* StateName ∈ S
* RoleName ∈ R
* RoleName ∈ valid_roles(StateName)

Otherwise → validation error.

---

## 4.3 Sequence Validity

For each sequence step:

```
ActionName: s1[r1] -> s2[r2]
```

The following must hold:

1. Both state-role combinations are valid:
   ```
   (s1, r1) ∈ V
   (s2, r2) ∈ V
   ```

2. For consecutive steps N and N+1:
   ```
   destination(step_N) = source(step_N+1)
   ```

---

## 4.4 Role Defaulting Rule (Option B)

If a state does not explicitly declare compatible roles:

```
valid_roles(s) = R
```

This reduces verbosity while preserving type safety.

---

## 4.5 Group Validity

For each group declaration:

```
group G { s1, s2, ..., sN }
```

The following must hold:

1. All referenced states must be defined: `s1, s2, ..., sN ∈ S`
2. Group must contain at least one state: `N ≥ 1`
3. A state may appear in multiple groups

Groups are organizational metadata. They do not affect the formal model `M = (S, R, V, Q)` but provide structural annotations for visualization (DOT subgraph clusters) and analysis.

---

# 5. Internal Representation (Implementation Guidance)

## 5.1 Recommended Core Structures

In memory, represent:

### Roles

```
Set<Role>
```

### States

```
State {
    name: String
    allowedRoles: Set<Role> | null (null = all roles)
}
```

### Nodes (V)

Do not store explicitly.

Generate implicitly as needed:

```
(s, r) valid if r ∈ valid_roles(s)
```

---

### Transitions

```
Transition {
    name: String
    cases: List<TransitionCase>
}

TransitionCase {
    from: (State, Role)
    to:   (State, Role)
}
```

---

### Graph View

For analysis, derive:

```
Node = (State, Role)
Edge = (Node, ActionName, Node)
```

---

# 6. Validation Strategy

Validation should occur in this order:

1. Parse syntax
2. Register roles
3. Register states
4. Validate state role constraints
5. Validate sequences (including chain connectivity)

Validation complexity is linear in number of transitions.

---

# 7. Implementation Roadmap

### Phase 1 — Parser

* Write grammar
* Generate AST
* No semantics yet

### Phase 2 — Semantic Analyzer

* Build symbol tables
* Validate references
* Enforce constraints

### Phase 3 — Graph Builder

* Convert to graph representation
* Enable reachability checks

### Phase 4 — CLI Tooling

* Validate file
* Print graph
* Export JSON
* Detect unreachable states

---

# 8. Future Extensions (Not in V1)

* Multi-participant modeling
* Action categories
* Phases
* Guarded transitions
* Determinism enforcement
* Composition of systems
* DSL imports

---

# 9. Core Philosophy

This DSL models:

* Structural martial knowledge
* Positional logic
* Transitions of configuration

It does NOT model:

* Violence
* Damage
* Scoring
* Strategy AI

It is a formal structural language for positional systems.
