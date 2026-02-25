//! Abstract Syntax Tree (AST) for the Martial DSL
//!
//! These types represent the parsed structure of martial system declarations.
//! Multiple `.martial` files can be loaded from a directory and combined.

/// A parsed martial file contains a list of declarations
#[derive(Debug, Clone, PartialEq)]
pub struct MartialFile {
    pub declarations: Vec<Declaration>,
}

/// A declaration at the top level of a file
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Roles(RolesDecl),
    State(State),
    Sequence(Sequence),
    Group(GroupDecl),
}

/// A roles declaration
///
/// Example: `roles { Top, Bottom, Neutral }`
/// Can appear in multiple files and will be merged.
#[derive(Debug, Clone, PartialEq)]
pub struct RolesDecl {
    pub roles: Vec<String>,
}

/// A state declaration
///
/// Example: `state Mount roles { Top, Bottom }`
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub name: String,
    /// Optional role restrictions. If None, all roles are valid.
    pub allowed_roles: Option<Vec<String>>,
}

/// A state reference with a role
///
/// Example: `Mount[Top]`
#[derive(Debug, Clone, PartialEq)]
pub struct StateRef {
    pub state: String,
    pub role: String,
}

/// A sequence declaration - ordered progression of actions
///
/// Example:
/// ```text
/// sequence GuardPass:
///     Stack: OpenGuard[Top] -> HalfGuard[Top]
///     KneeSlice: HalfGuard[Top] -> SideControl[Top]
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub name: String,
    pub steps: Vec<SequenceStep>,
}

/// A single step within a sequence - an action with explicit transition
///
/// Example: `KneeCut: Headquarters[Top] -> SideControl[Top]`
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceStep {
    pub action_name: String,
    pub from: StateRef,
    pub to: StateRef,
}

/// A group declaration - organizational clustering of related states
///
/// Example:
/// ```text
/// group ClosedGuardFamily {
///     ClosedGuard, WilliamsGuard, RubberGuard
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GroupDecl {
    pub name: String,
    pub states: Vec<String>,
}
