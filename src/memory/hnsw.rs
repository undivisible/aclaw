//! HNSW vector index on top of Surreal-backed embeddings.
//! Pure Rust implementation — no external deps.
//! Hierarchical Navigable Small World graph for ANN search.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswNode {
    pub id: usize,
    pub vector: Vec<f32>,
    pub label: String,
    pub layers: Vec<Vec<usize>>, // adjacency list per layer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswIndex {
    pub nodes: Vec<HnswNode>,
    pub entry_point: Option<usize>,
    pub max_layers: usize,
    pub m: usize,      // max connections per layer
    pub m_max0: usize, // max connections at layer 0
    pub ef_construction: usize,
    pub ml: f64, // normalization factor
}

#[derive(Debug, Clone, PartialEq)]
struct Candidate {
    dist: f32,
    id: usize,
}

impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .dist
            .partial_cmp(&self.dist)
            .unwrap_or(Ordering::Equal)
    }
}

impl HnswIndex {
    pub fn new(m: usize, ef_construction: usize) -> Self {
        let m = m.max(2);
        Self {
            nodes: Vec::new(),
            entry_point: None,
            max_layers: 16,
            m,
            m_max0: m * 2,
            ef_construction,
            ml: 1.0 / (m as f64).ln(),
        }
    }

    fn random_level(&self) -> usize {
        let r: f64 = rand_f64();
        let level = (-r.ln() * self.ml).floor() as usize;
        level.min(self.max_layers - 1)
    }

    fn distance(a: &[f32], b: &[f32]) -> f32 {
        1.0 - cosine_similarity(a, b)
    }

    pub fn insert(&mut self, vector: Vec<f32>, label: String) -> usize {
        let id = self.nodes.len();
        let level = self.random_level();
        let mut neighbors_by_layer = Vec::new();

        if let Some(ep) = self.entry_point {
            let mut curr_ep = vec![ep];
            let max_layer = self.nodes[ep].layers.len() - 1;

            // Traverse from top down to level+1
            for lc in (level + 1..=max_layer).rev() {
                curr_ep = self.search_layer(&vector, &curr_ep, 1, lc);
            }

            // Collect neighbors for each layer from level down to 0
            for lc in (0..=level.min(max_layer)).rev() {
                let candidates = self.search_layer(&vector, &curr_ep, self.ef_construction, lc);
                let neighbors = self.select_neighbors(&vector, &candidates, self.m);
                neighbors_by_layer.push((lc, neighbors.clone()));
                curr_ep = candidates;
            }

            if level > max_layer {
                self.entry_point = Some(id);
            }
        } else {
            self.entry_point = Some(id);
        }

        // Create the node with its neighbors
        let mut layers = vec![Vec::new(); level + 1];
        for (lc, neighbors) in &neighbors_by_layer {
            layers[*lc] = neighbors.clone();
        }

        let node = HnswNode {
            id,
            vector: vector.clone(),
            label,
            layers,
        };

        // Push the node so 'id' is now valid
        self.nodes.push(node);

        // Update neighbors' back-connections to point to the new 'id'
        for (lc, neighbors) in neighbors_by_layer {
            let m_max = if lc == 0 { self.m_max0 } else { self.m };
            for neighbor_id in neighbors {
                if !self.nodes[neighbor_id].layers[lc].contains(&id) {
                    self.nodes[neighbor_id].layers[lc].push(id);
                    if self.nodes[neighbor_id].layers[lc].len() > m_max {
                        let nv = self.nodes[neighbor_id].vector.clone();
                        let layer_copy = self.nodes[neighbor_id].layers[lc].clone();
                        let kept = self.select_neighbors_from_ids(&nv, &layer_copy, m_max);
                        self.nodes[neighbor_id].layers[lc] = kept;
                    }
                }
            }
        }

        id
    }

    fn search_layer(
        &self,
        query: &[f32],
        entry_points: &[usize],
        ef: usize,
        layer: usize,
    ) -> Vec<usize> {
        let mut visited: HashSet<usize> = entry_points.iter().cloned().collect();
        let mut candidates: BinaryHeap<Candidate> = BinaryHeap::new();
        let mut results: BinaryHeap<std::cmp::Reverse<Candidate>> = BinaryHeap::new();

        for &ep in entry_points {
            let d = Self::distance(query, &self.nodes[ep].vector);
            candidates.push(Candidate { dist: d, id: ep });
            results.push(std::cmp::Reverse(Candidate { dist: d, id: ep }));
        }

        while let Some(curr) = candidates.pop() {
            let worst_result = results.peek().map(|r| r.0.dist).unwrap_or(f32::MAX);
            if curr.dist > worst_result && results.len() >= ef {
                break;
            }

            if curr.id < self.nodes.len() && layer < self.nodes[curr.id].layers.len() {
                for &neighbor in &self.nodes[curr.id].layers[layer].clone() {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        let d = Self::distance(query, &self.nodes[neighbor].vector);
                        let worst = results.peek().map(|r| r.0.dist).unwrap_or(f32::MAX);
                        if d < worst || results.len() < ef {
                            candidates.push(Candidate {
                                dist: d,
                                id: neighbor,
                            });
                            results.push(std::cmp::Reverse(Candidate {
                                dist: d,
                                id: neighbor,
                            }));
                            if results.len() > ef {
                                results.pop();
                            }
                        }
                    }
                }
            }
        }

        results.into_iter().map(|r| r.0.id).collect()
    }

    fn select_neighbors(&self, query: &[f32], candidates: &[usize], m: usize) -> Vec<usize> {
        self.select_neighbors_from_ids(query, candidates, m)
    }

    fn select_neighbors_from_ids(
        &self,
        query: &[f32],
        candidates: &[usize],
        m: usize,
    ) -> Vec<usize> {
        let mut scored: Vec<(f32, usize)> = candidates
            .iter()
            .map(|&id| (Self::distance(query, &self.nodes[id].vector), id))
            .collect();
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        scored.into_iter().take(m).map(|(_, id)| id).collect()
    }

    pub fn search(&self, query: &[f32], k: usize) -> Vec<(usize, f32, &str)> {
        let ep = match self.entry_point {
            Some(ep) => ep,
            None => return Vec::new(),
        };

        let mut curr_ep = vec![ep];
        let max_layer = self.nodes[ep].layers.len().saturating_sub(1);

        for lc in (1..=max_layer).rev() {
            curr_ep = self.search_layer(query, &curr_ep, 1, lc);
        }

        let candidates = self.search_layer(query, &curr_ep, k.max(self.ef_construction), 0);

        let mut results: Vec<(f32, usize)> = candidates
            .iter()
            .map(|&id| (Self::distance(query, &self.nodes[id].vector), id))
            .collect();
        results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        results.truncate(k);

        results
            .into_iter()
            .map(|(dist, id)| (id, 1.0 - dist, self.nodes[id].label.as_str()))
            .collect()
    }

    pub fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(data)?)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let ma: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if ma == 0.0 || mb == 0.0 {
        return 0.0;
    }
    dot / (ma * mb)
}

// Simple LCG-based random (no dep needed, not crypto)
fn rand_f64() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    static SEED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let mut s = SEED.load(std::sync::atomic::Ordering::Relaxed);
    if s == 0 {
        s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
    }
    s = s
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    SEED.store(s, std::sync::atomic::Ordering::Relaxed);
    (s >> 11) as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_basic() {
        let mut index = HnswIndex::new(8, 50);
        for i in 0..100 {
            let v: Vec<f32> = (0..128).map(|j| ((i * 128 + j) as f32).sin()).collect();
            index.insert(v, format!("item_{}", i));
        }
        let query: Vec<f32> = (0..128).map(|j| (j as f32).sin()).collect();
        let results = index.search(&query, 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].2, "item_0"); // should find itself
    }
}
