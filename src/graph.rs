//! Graph representation for the Martial DSL
//!
//! Converts a validated martial system into a directed graph structure
//! for analysis and visualization.

use crate::semantic::MartialSystem;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// A node in the martial graph represents a (State, Role) combination
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Node {
    pub state: String,
    pub role: String,
}

impl Node {
    pub fn new(state: String, role: String) -> Self {
        Node { state, role }
    }
    
    pub fn id(&self) -> String {
        format!("{}[{}]", self.state, self.role)
    }
}

/// An edge in the martial graph represents an action/transition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub from: Node,
    pub to: Node,
    pub action: String,
    pub sequence: String,
}

/// A directed graph representing the martial system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MartialGraph {
    pub system_name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl MartialGraph {
    /// Build a graph from a validated martial system
    pub fn from_system(system: &MartialSystem) -> Self {
        let mut nodes_set = HashSet::new();
        let mut edges = Vec::new();

        // Extract nodes and edges from all sequences
        for (seq_name, sequence) in &system.sequences {
            for step in &sequence.steps {
                let from_node = Node::new(step.from.state.clone(), step.from.role.clone());
                let to_node = Node::new(step.to.state.clone(), step.to.role.clone());

                nodes_set.insert(from_node.clone());
                nodes_set.insert(to_node.clone());

                edges.push(Edge {
                    from: from_node,
                    to: to_node,
                    action: step.action_name.clone(),
                    sequence: seq_name.clone(),
                });
            }
        }

        let mut nodes: Vec<Node> = nodes_set.into_iter().collect();
        nodes.sort_by(|a, b| {
            let cmp = a.state.cmp(&b.state);
            if cmp == std::cmp::Ordering::Equal {
                a.role.cmp(&b.role)
            } else {
                cmp
            }
        });

        MartialGraph {
            system_name: system.name.clone(),
            nodes,
            edges,
        }
    }

    /// Get all nodes reachable from a given node
    pub fn reachable_from(&self, start: &Node) -> HashSet<Node> {
        let mut reachable = HashSet::new();
        let mut to_visit = vec![start.clone()];
        
        while let Some(current) = to_visit.pop() {
            if !reachable.insert(current.clone()) {
                continue; // Already visited
            }
            
            // Find all edges from current node
            for edge in &self.edges {
                if edge.from == current && !reachable.contains(&edge.to) {
                    to_visit.push(edge.to.clone());
                }
            }
        }
        
        reachable
    }

    /// Find all unreachable nodes (nodes with no incoming edges and not starting points)
    pub fn find_unreachable_nodes(&self) -> Vec<Node> {
        if self.nodes.is_empty() {
            return Vec::new();
        }

        // Nodes that have incoming edges or are sources
        let mut reachable = HashSet::new();
        
        // Add all source nodes (nodes with outgoing but possibly no incoming edges)
        for edge in &self.edges {
            reachable.insert(edge.from.clone());
        }
        
        // For each source, find all reachable nodes
        let sources: Vec<Node> = reachable.iter().cloned().collect();
        for source in sources {
            let reached = self.reachable_from(&source);
            for node in reached {
                reachable.insert(node);
            }
        }

        // Find nodes not in reachable set
        self.nodes
            .iter()
            .filter(|node| !reachable.contains(node))
            .cloned()
            .collect()
    }

    /// Export as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export as DOT format for Graphviz
    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str(&format!("digraph \"{}\" {{\n", self.system_name));
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=rounded];\n\n");

        // Add nodes
        for node in &self.nodes {
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n[{}]\"];\n",
                node.id(),
                node.state,
                node.role
            ));
        }

        dot.push_str("\n");

        // Add edges
        for edge in &self.edges {
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                edge.from.id(),
                edge.to.id(),
                edge.action
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Get statistics about the graph
    pub fn statistics(&self) -> GraphStatistics {
        let mut in_degree: HashMap<&Node, usize> = HashMap::new();
        let mut out_degree: HashMap<&Node, usize> = HashMap::new();
        let mut self_loops = 0;

        for edge in &self.edges {
            *out_degree.entry(&edge.from).or_insert(0) += 1;
            *in_degree.entry(&edge.to).or_insert(0) += 1;
            
            if edge.from == edge.to {
                self_loops += 1;
            }
        }

        let source_nodes = self.nodes.iter()
            .filter(|n| in_degree.get(n).unwrap_or(&0) == &0 && out_degree.get(n).unwrap_or(&0) > &0)
            .cloned()
            .collect();

        let sink_nodes = self.nodes.iter()
            .filter(|n| out_degree.get(n).unwrap_or(&0) == &0 && in_degree.get(n).unwrap_or(&0) > &0)
            .cloned()
            .collect();

        let isolated_nodes = self.nodes.iter()
            .filter(|n| in_degree.get(n).unwrap_or(&0) == &0 && out_degree.get(n).unwrap_or(&0) == &0)
            .cloned()
            .collect();

        GraphStatistics {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            self_loops,
            source_nodes,
            sink_nodes,
            isolated_nodes,
        }
    }
}

/// Graph statistics
#[derive(Debug, Clone)]
pub struct GraphStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub self_loops: usize,
    pub source_nodes: Vec<Node>,
    pub sink_nodes: Vec<Node>,
    pub isolated_nodes: Vec<Node>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use std::collections::HashSet;

    fn make_test_system() -> MartialSystem {
        let mut roles = HashSet::new();
        roles.insert("Top".to_string());
        roles.insert("Bottom".to_string());

        let mut states = HashMap::new();
        states.insert(
            "Mount".to_string(),
            State {
                name: "Mount".to_string(),
                allowed_roles: None,
            },
        );
        states.insert(
            "Guard".to_string(),
            State {
                name: "Guard".to_string(),
                allowed_roles: None,
            },
        );

        let mut sequences = HashMap::new();
        sequences.insert(
            "Escape".to_string(),
            Sequence {
                name: "Escape".to_string(),
                steps: vec![
                    SequenceStep {
                        action_name: "Shrimp".to_string(),
                        from: StateRef {
                            state: "Mount".to_string(),
                            role: "Bottom".to_string(),
                        },
                        to: StateRef {
                            state: "Guard".to_string(),
                            role: "Bottom".to_string(),
                        },
                    },
                ],
            },
        );

        MartialSystem {
            name: "BJJ".to_string(),
            roles,
            states,
            sequences,
        }
    }

    #[test]
    fn test_graph_creation() {
        let system = make_test_system();
        let graph = MartialGraph::from_system(&system);

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.system_name, "BJJ");
    }

    #[test]
    fn test_reachability() {
        let system = make_test_system();
        let graph = MartialGraph::from_system(&system);

        let start = Node::new("Mount".to_string(), "Bottom".to_string());
        let reachable = graph.reachable_from(&start);

        assert_eq!(reachable.len(), 2); // Mount[Bottom] and Guard[Bottom]
        assert!(reachable.contains(&Node::new("Guard".to_string(), "Bottom".to_string())));
    }

    #[test]
    fn test_statistics() {
        let system = make_test_system();
        let graph = MartialGraph::from_system(&system);
        let stats = graph.statistics();

        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.edge_count, 1);
        assert_eq!(stats.self_loops, 0);
        assert_eq!(stats.source_nodes.len(), 1);
        assert_eq!(stats.sink_nodes.len(), 1);
    }

    #[test]
    fn test_dot_export() {
        let system = make_test_system();
        let graph = MartialGraph::from_system(&system);
        let dot = graph.to_dot();

        assert!(dot.contains("digraph \"BJJ\""));
        assert!(dot.contains("Mount[Bottom]"));
        assert!(dot.contains("Guard[Bottom]"));
        assert!(dot.contains("Shrimp"));
    }

    #[test]
    fn test_json_export() {
        let system = make_test_system();
        let graph = MartialGraph::from_system(&system);
        let json = graph.to_json().unwrap();

        assert!(json.contains("BJJ"));
        assert!(json.contains("Mount"));
        assert!(json.contains("Shrimp"));
    }
}
