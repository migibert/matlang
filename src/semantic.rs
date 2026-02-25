//! Semantic analysis for the Martial DSL
//!
//! Validates a martial system across multiple files:
//! - Collects and merges roles from all files
//! - Validates state-role combinations
//! - Validates sequence step connectivity

use crate::ast::*;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Semantic validation error
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticError {
    pub message: String,
    pub context: String,
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Semantic error in {}: {}", self.context, self.message)
    }
}

/// A validated martial system
#[derive(Debug, Clone)]
pub struct MartialSystem {
    pub name: String,
    pub roles: HashSet<String>,
    pub states: HashMap<String, State>,
    pub sequences: HashMap<String, Sequence>,
    pub groups: HashMap<String, Vec<String>>,
}

/// Semantic validator
pub struct SemanticValidator {
    /// All declared roles (merged from all files)
    roles: HashSet<String>,
    /// All declared states
    states: HashMap<String, State>,
    /// All declared sequences
    sequences: HashMap<String, Sequence>,
    /// All declared groups
    groups: HashMap<String, Vec<String>>,
}

impl SemanticValidator {
    /// Create a new validator
    pub fn new() -> Self {
        SemanticValidator {
            roles: HashSet::new(),
            states: HashMap::new(),
            sequences: HashMap::new(),
            groups: HashMap::new(),
        }
    }

    /// Add declarations from a parsed file
    pub fn add_file(&mut self, file: MartialFile) -> Result<(), SemanticError> {
        for declaration in file.declarations {
            match declaration {
                Declaration::Roles(roles_decl) => {
                    self.add_roles(roles_decl)?;
                }
                Declaration::State(state) => {
                    self.add_state(state)?;
                }
                Declaration::Sequence(sequence) => {
                    self.add_sequence(sequence)?;
                }
                Declaration::Group(group) => {
                    self.add_group(group)?;
                }
            }
        }
        Ok(())
    }

    /// Add roles (can be called multiple times, roles are merged)
    fn add_roles(&mut self, roles_decl: RolesDecl) -> Result<(), SemanticError> {
        for role in roles_decl.roles {
            if role.is_empty() {
                return Err(SemanticError {
                    message: "Role name cannot be empty".to_string(),
                    context: "roles declaration".to_string(),
                });
            }
            self.roles.insert(role);
        }
        Ok(())
    }

    /// Add a state
    fn add_state(&mut self, state: State) -> Result<(), SemanticError> {
        if state.name.is_empty() {
            return Err(SemanticError {
                message: "State name cannot be empty".to_string(),
                context: "state declaration".to_string(),
            });
        }

        if self.states.contains_key(&state.name) {
            return Err(SemanticError {
                message: format!("State '{}' is already defined", state.name),
                context: format!("state {}", state.name),
            });
        }

        self.states.insert(state.name.clone(), state);
        Ok(())
    }

    /// Add a sequence
    fn add_sequence(&mut self, sequence: Sequence) -> Result<(), SemanticError> {
        if sequence.name.is_empty() {
            return Err(SemanticError {
                message: "Sequence name cannot be empty".to_string(),
                context: "sequence declaration".to_string(),
            });
        }

        if self.sequences.contains_key(&sequence.name) {
            return Err(SemanticError {
                message: format!("Sequence '{}' is already defined", sequence.name),
                context: format!("sequence {}", sequence.name),
            });
        }

        self.sequences.insert(sequence.name.clone(), sequence);
        Ok(())
    }

    /// Add a group
    fn add_group(&mut self, group: GroupDecl) -> Result<(), SemanticError> {
        if group.name.is_empty() {
            return Err(SemanticError {
                message: "Group name cannot be empty".to_string(),
                context: "group declaration".to_string(),
            });
        }

        if self.groups.contains_key(&group.name) {
            return Err(SemanticError {
                message: format!("Group '{}' is already defined", group.name),
                context: format!("group {}", group.name),
            });
        }

        self.groups.insert(group.name, group.states);
        Ok(())
    }

    /// Validate the entire system
    pub fn validate(self, system_name: String) -> Result<MartialSystem, SemanticError> {
        // Check that we have at least one role
        if self.roles.is_empty() {
            return Err(SemanticError {
                message: "No roles defined. At least one role declaration is required.".to_string(),
                context: system_name,
            });
        }

        // Validate states
        self.validate_states()?;

        // Validate sequences
        self.validate_sequences()?;

        // Validate groups
        self.validate_groups()?;

        Ok(MartialSystem {
            name: system_name,
            roles: self.roles,
            states: self.states,
            sequences: self.sequences,
            groups: self.groups,
        })
    }

    /// Validate all states
    fn validate_states(&self) -> Result<(), SemanticError> {
        for (state_name, state) in &self.states {
            if let Some(allowed_roles) = &state.allowed_roles {
                // Check that all allowed roles exist
                for role in allowed_roles {
                    if !self.roles.contains(role) {
                        return Err(SemanticError {
                            message: format!(
                                "Role '{}' is not defined. Available roles: {}",
                                role,
                                self.roles.iter().cloned().collect::<Vec<_>>().join(", ")
                            ),
                            context: format!("state {}", state_name),
                        });
                    }
                }

                // Check for duplicate roles
                let mut seen = HashSet::new();
                for role in allowed_roles {
                    if !seen.insert(role) {
                        return Err(SemanticError {
                            message: format!("Role '{}' appears multiple times", role),
                            context: format!("state {}", state_name),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate all groups
    fn validate_groups(&self) -> Result<(), SemanticError> {
        for (group_name, states) in &self.groups {
            if states.is_empty() {
                return Err(SemanticError {
                    message: "Group must contain at least one state".to_string(),
                    context: format!("group {}", group_name),
                });
            }

            for state_name in states {
                if !self.states.contains_key(state_name) {
                    return Err(SemanticError {
                        message: format!(
                            "State '{}' is not defined. Available states: {}",
                            state_name,
                            self.states.keys().cloned().collect::<Vec<_>>().join(", ")
                        ),
                        context: format!("group {}", group_name),
                    });
                }
            }
        }
        Ok(())
    }

    /// Validate all sequences
    fn validate_sequences(&self) -> Result<(), SemanticError> {
        for (seq_name, sequence) in &self.sequences {
            if sequence.steps.is_empty() {
                return Err(SemanticError {
                    message: "Sequence must have at least one step".to_string(),
                    context: format!("sequence {}", seq_name),
                });
            }

            // Validate each step
            for (i, step) in sequence.steps.iter().enumerate() {
                let step_context = format!("sequence {} step {} ({})", seq_name, i + 1, step.action_name);

                // Validate 'from' state reference
                self.validate_state_ref(&step.from, &step_context)?;

                // Validate 'to' state reference
                self.validate_state_ref(&step.to, &step_context)?;

                // Validate chain connectivity (step N's 'to' must equal step N+1's 'from')
                if i > 0 {
                    let prev_step = &sequence.steps[i - 1];
                    if prev_step.to.state != step.from.state || prev_step.to.role != step.from.role {
                        return Err(SemanticError {
                            message: format!(
                                "Step chain is broken: previous step ends at {}[{}], but this step starts at {}[{}]",
                                prev_step.to.state,
                                prev_step.to.role,
                                step.from.state,
                                step.from.role
                            ),
                            context: step_context,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate a state reference
    fn validate_state_ref(&self, state_ref: &StateRef, context: &str) -> Result<(), SemanticError> {
        // Check that state exists
        let state = self.states.get(&state_ref.state).ok_or_else(|| SemanticError {
            message: format!(
                "State '{}' is not defined. Available states: {}",
                state_ref.state,
                self.states.keys().cloned().collect::<Vec<_>>().join(", ")
            ),
            context: context.to_string(),
        })?;

        // Check that role exists
        if !self.roles.contains(&state_ref.role) {
            return Err(SemanticError {
                message: format!(
                    "Role '{}' is not defined. Available roles: {}",
                    state_ref.role,
                    self.roles.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
                context: context.to_string(),
            });
        }

        // Check that role is allowed for this state
        if let Some(allowed_roles) = &state.allowed_roles {
            if !allowed_roles.contains(&state_ref.role) {
                return Err(SemanticError {
                    message: format!(
                        "Role '{}' is not allowed for state '{}'. Allowed roles: {}",
                        state_ref.role,
                        state_ref.state,
                        allowed_roles.join(", ")
                    ),
                    context: context.to_string(),
                });
            }
        }
        // If no allowed_roles, all roles are valid (per spec)

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_roles(roles: Vec<&str>) -> RolesDecl {
        RolesDecl {
            roles: roles.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    fn make_state(name: &str, allowed_roles: Option<Vec<&str>>) -> State {
        State {
            name: name.to_string(),
            allowed_roles: allowed_roles.map(|r| r.into_iter().map(|s| s.to_string()).collect()),
        }
    }

    fn make_state_ref(state: &str, role: &str) -> StateRef {
        StateRef {
            state: state.to_string(),
            role: role.to_string(),
        }
    }

    #[test]
    fn test_merge_roles() {
        let mut validator = SemanticValidator::new();
        validator.add_roles(make_roles(vec!["Top", "Bottom"])).unwrap();
        validator.add_roles(make_roles(vec!["Neutral"])).unwrap();

        assert_eq!(validator.roles.len(), 3);
        assert!(validator.roles.contains("Top"));
        assert!(validator.roles.contains("Bottom"));
        assert!(validator.roles.contains("Neutral"));
    }

    #[test]
    fn test_duplicate_state() {
        let mut validator = SemanticValidator::new();
        validator.add_state(make_state("Mount", None)).unwrap();
        let result = validator.add_state(make_state("Mount", None));

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("already defined"));
    }

    #[test]
    fn test_state_with_undefined_role() {
        let mut validator = SemanticValidator::new();
        validator.add_roles(make_roles(vec!["Top"])).unwrap();
        validator.add_state(make_state("Mount", Some(vec!["Top", "Bottom"]))).unwrap();

        let result = validator.validate("test".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Role 'Bottom' is not defined"));
    }

    #[test]
    fn test_sequence_with_undefined_state() {
        let mut validator = SemanticValidator::new();
        validator.add_roles(make_roles(vec!["Top"])).unwrap();
        validator.add_state(make_state("Mount", None)).unwrap();

        let sequence = Sequence {
            name: "Test".to_string(),
            steps: vec![SequenceStep {
                action_name: "Move".to_string(),
                from: make_state_ref("Mount", "Top"),
                to: make_state_ref("Guard", "Top"),
            }],
        };
        validator.add_sequence(sequence).unwrap();

        let result = validator.validate("test".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("State 'Guard' is not defined"));
    }

    #[test]
    fn test_sequence_chain_validation() {
        let mut validator = SemanticValidator::new();
        validator.add_roles(make_roles(vec!["Top", "Bottom"])).unwrap();
        validator.add_state(make_state("A", None)).unwrap();
        validator.add_state(make_state("B", None)).unwrap();
        validator.add_state(make_state("C", None)).unwrap();

        // Chain with broken link
        let sequence = Sequence {
            name: "Test".to_string(),
            steps: vec![
                SequenceStep {
                    action_name: "Move1".to_string(),
                    from: make_state_ref("A", "Top"),
                    to: make_state_ref("B", "Top"),
                },
                SequenceStep {
                    action_name: "Move2".to_string(),
                    from: make_state_ref("C", "Top"), // Should be B[Top]
                    to: make_state_ref("A", "Top"),
                },
            ],
        };
        validator.add_sequence(sequence).unwrap();

        let result = validator.validate("test".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("chain is broken"));
    }

    #[test]
    fn test_valid_system() {
        let mut validator = SemanticValidator::new();
        validator.add_roles(make_roles(vec!["Top", "Bottom"])).unwrap();
        validator.add_state(make_state("Mount", Some(vec!["Top", "Bottom"]))).unwrap();
        validator.add_state(make_state("Guard", Some(vec!["Top", "Bottom"]))).unwrap();

        let sequence = Sequence {
            name: "Escape".to_string(),
            steps: vec![
                SequenceStep {
                    action_name: "Shrimp".to_string(),
                    from: make_state_ref("Mount", "Bottom"),
                    to: make_state_ref("Guard", "Bottom"),
                },
            ],
        };
        validator.add_sequence(sequence).unwrap();

        let result = validator.validate("BJJ".to_string());
        assert!(result.is_ok());
        let system = result.unwrap();
        assert_eq!(system.name, "BJJ");
        assert_eq!(system.roles.len(), 2);
        assert_eq!(system.states.len(), 2);
        assert_eq!(system.sequences.len(), 1);
    }

    #[test]
    fn test_valid_group() {
        let mut validator = SemanticValidator::new();
        validator.add_roles(make_roles(vec!["Top", "Bottom"])).unwrap();
        validator.add_state(make_state("Mount", None)).unwrap();
        validator.add_state(make_state("SideControl", None)).unwrap();
        validator.add_state(make_state("Guard", None)).unwrap();

        let group = GroupDecl {
            name: "TopPositions".to_string(),
            states: vec!["Mount".to_string(), "SideControl".to_string()],
        };
        validator.add_group(group).unwrap();

        let result = validator.validate("Test".to_string());
        assert!(result.is_ok());
        let system = result.unwrap();
        assert_eq!(system.groups.len(), 1);
        assert!(system.groups.contains_key("TopPositions"));
        assert_eq!(system.groups["TopPositions"], vec!["Mount", "SideControl"]);
    }

    #[test]
    fn test_group_with_undefined_state() {
        let mut validator = SemanticValidator::new();
        validator.add_roles(make_roles(vec!["Top"])).unwrap();
        validator.add_state(make_state("Mount", None)).unwrap();

        let group = GroupDecl {
            name: "Bad".to_string(),
            states: vec!["Mount".to_string(), "NonExistent".to_string()],
        };
        validator.add_group(group).unwrap();

        let result = validator.validate("Test".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("State 'NonExistent' is not defined"));
    }

    #[test]
    fn test_duplicate_group() {
        let mut validator = SemanticValidator::new();

        let group1 = GroupDecl {
            name: "Guards".to_string(),
            states: vec!["A".to_string()],
        };
        let group2 = GroupDecl {
            name: "Guards".to_string(),
            states: vec!["B".to_string()],
        };
        validator.add_group(group1).unwrap();
        let result = validator.add_group(group2);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("already defined"));
    }
}
