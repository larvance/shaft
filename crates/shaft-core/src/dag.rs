use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::registry::{get_all_conversions, normalize_format};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversionRoute {
    pub source: String,
    pub target: String,
    pub steps: Vec<String>,
    pub total_cost: f32,
    pub is_direct: bool,
}

#[derive(Clone, PartialEq)]
struct State {
    cost: f32,
    node: String,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap ordering based on cost
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionDAG {
    adjacency: HashMap<String, Vec<(String, f32)>>,
    nodes: HashSet<String>,
}

impl Default for ConversionDAG {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversionDAG {
    pub fn new() -> Self {
        let mut dag = Self {
            adjacency: HashMap::new(),
            nodes: HashSet::new(),
        };

        let raw_matrix = get_all_conversions();
        for (source, targets) in raw_matrix {
            let norm_src = normalize_format(&source);
            dag.nodes.insert(norm_src.clone());

            for target in targets {
                let norm_tgt = normalize_format(&target);
                dag.nodes.insert(norm_tgt.clone());
                dag.add_edge(&norm_src, &norm_tgt, 1.0);
            }
        }

        dag
    }

    pub fn add_edge(&mut self, source: &str, target: &str, weight: f32) {
        let s = normalize_format(source);
        let t = normalize_format(target);
        self.nodes.insert(s.clone());
        self.nodes.insert(t.clone());

        self.adjacency
            .entry(s)
            .or_default()
            .push((t, weight));
    }

    pub fn can_convert(&self, source: &str, target: &str) -> bool {
        self.find_shortest_route(source, target).is_some()
    }

    pub fn is_direct_conversion(&self, source: &str, target: &str) -> bool {
        let s = normalize_format(source);
        let t = normalize_format(target);
        if let Some(neighbors) = self.adjacency.get(&s) {
            neighbors.iter().any(|(next, _)| next == &t)
        } else {
            false
        }
    }

    pub fn find_shortest_route(&self, source: &str, target: &str) -> Option<ConversionRoute> {
        let start = normalize_format(source);
        let goal = normalize_format(target);

        if start == goal {
            return Some(ConversionRoute {
                source: start.clone(),
                target: goal.clone(),
                steps: vec![start],
                total_cost: 0.0,
                is_direct: true,
            });
        }

        if !self.nodes.contains(&start) || !self.nodes.contains(&goal) {
            return None;
        }

        let mut distances: HashMap<String, f32> = HashMap::new();
        let mut previous: HashMap<String, String> = HashMap::new();
        let mut heap: BinaryHeap<State> = BinaryHeap::new();

        distances.insert(start.clone(), 0.0);
        heap.push(State {
            cost: 0.0,
            node: start.clone(),
        });

        while let Some(State { cost, node }) = heap.pop() {
            if node == goal {
                // Reconstruct path
                let mut path = vec![goal.clone()];
                let mut curr = goal.clone();

                while let Some(prev) = previous.get(&curr) {
                    path.push(prev.clone());
                    curr = prev.clone();
                }

                path.reverse();
                let is_direct = path.len() == 2;

                return Some(ConversionRoute {
                    source: start,
                    target: goal,
                    steps: path,
                    total_cost: cost,
                    is_direct,
                });
            }

            if let Some(&best) = distances.get(&node) {
                if cost > best {
                    continue;
                }
            }

            if let Some(neighbors) = self.adjacency.get(&node) {
                for (next_node, weight) in neighbors {
                    let next_cost = cost + weight;
                    let is_better = match distances.get(next_node) {
                        Some(&curr_best) => next_cost < curr_best,
                        None => true,
                    };

                    if is_better {
                        distances.insert(next_node.clone(), next_cost);
                        previous.insert(next_node.clone(), node.clone());
                        heap.push(State {
                            cost: next_cost,
                            node: next_node.clone(),
                        });
                    }
                }
            }
        }

        None
    }

    pub fn get_direct_targets(&self, source: &str) -> Vec<String> {
        let s = normalize_format(source);
        if let Some(neighbors) = self.adjacency.get(&s) {
            neighbors.iter().map(|(next, _)| next.clone()).collect()
        } else {
            vec![]
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_initialization() {
        let dag = ConversionDAG::new();
        assert!(dag.node_count() >= 35, "Expected >=35 format nodes, got {}", dag.node_count());
        assert!(dag.edge_count() >= 272, "Expected >=272 edges, got {}", dag.edge_count());
    }

    #[test]
    fn test_direct_image_route() {
        let dag = ConversionDAG::new();
        let route = dag.find_shortest_route("png", "webp").expect("PNG to WebP route must exist");
        assert_eq!(route.steps, vec!["png", "webp"]);
        assert!(route.is_direct);
        assert_eq!(route.total_cost, 1.0);
    }

    #[test]
    fn test_audio_bidirectional_routes() {
        let dag = ConversionDAG::new();
        let audio = ["mp3", "m4a", "wav", "flac", "ogg", "opus", "aiff", "wma"];
        for src in audio {
            for tgt in audio {
                if src != tgt {
                    assert!(
                        dag.is_direct_conversion(src, tgt),
                        "Direct route from {} to {} must exist",
                        src,
                        tgt
                    );
                }
            }
        }
    }

    #[test]
    fn test_pdf_to_markdown() {
        let dag = ConversionDAG::new();
        let route = dag.find_shortest_route("pdf", "md").expect("PDF to Markdown route must exist");
        assert_eq!(route.steps, vec!["pdf", "md"]);
    }
}
