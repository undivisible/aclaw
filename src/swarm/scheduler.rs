//! Concurrency manager — lane-based scheduling, active delegation tracking,
//! limit enforcement, and deadlock detection.
//!
//! Lanes:
//! - Main: primary agent interactions
//! - Delegate: delegated tasks from other agents
//! - Cron: scheduled background tasks

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Execution lane types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lane {
    Main,
    Delegate,
    Cron,
}

impl std::fmt::Display for Lane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lane::Main => write!(f, "main"),
            Lane::Delegate => write!(f, "delegate"),
            Lane::Cron => write!(f, "cron"),
        }
    }
}

/// Lane configuration
#[derive(Debug, Clone)]
pub struct LaneConfig {
    pub max_concurrent: usize,
    pub priority: u8,
}

impl Default for LaneConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 1,
            priority: 0,
        }
    }
}

/// Active execution slot
#[derive(Debug, Clone)]
pub struct ExecutionSlot {
    pub slot_id: String,
    pub agent_id: String,
    pub lane: Lane,
    pub description: String,
    pub started_at: DateTime<Utc>,
    /// Optional: agent this task is waiting on
    pub waiting_on: Option<String>,
}

/// Concurrency scheduler
pub struct ConcurrencyScheduler {
    lane_configs: HashMap<Lane, LaneConfig>,
    active_slots: Arc<RwLock<Vec<ExecutionSlot>>>,
    /// Tracks which agents are waiting on which other agents (for deadlock detection)
    wait_graph: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for ConcurrencyScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl ConcurrencyScheduler {
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        configs.insert(
            Lane::Main,
            LaneConfig {
                max_concurrent: 3,
                priority: 2,
            },
        );
        configs.insert(
            Lane::Delegate,
            LaneConfig {
                max_concurrent: 5,
                priority: 1,
            },
        );
        configs.insert(
            Lane::Cron,
            LaneConfig {
                max_concurrent: 2,
                priority: 0,
            },
        );

        Self {
            lane_configs: configs,
            active_slots: Arc::new(RwLock::new(Vec::new())),
            wait_graph: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Configure a lane
    pub fn configure_lane(&mut self, lane: Lane, config: LaneConfig) {
        self.lane_configs.insert(lane, config);
    }

    /// Try to acquire an execution slot. Returns None if lane is full.
    pub async fn acquire_slot(
        &self,
        agent_id: &str,
        lane: Lane,
        description: &str,
    ) -> Option<String> {
        let config = self.lane_configs.get(&lane)?;
        let mut slots = self.active_slots.write().await;

        let lane_count = slots.iter().filter(|s| s.lane == lane).count();
        if lane_count >= config.max_concurrent {
            return None;
        }

        let slot_id = uuid::Uuid::new_v4().to_string();
        slots.push(ExecutionSlot {
            slot_id: slot_id.clone(),
            agent_id: agent_id.to_string(),
            lane,
            description: description.to_string(),
            started_at: Utc::now(),
            waiting_on: None,
        });

        Some(slot_id)
    }

    /// Release an execution slot
    pub async fn release_slot(&self, slot_id: &str) {
        let mut slots = self.active_slots.write().await;
        slots.retain(|s| s.slot_id != slot_id);

        // Clean up wait graph
        let mut graph = self.wait_graph.write().await;
        graph.retain(|_, v| v != slot_id);
    }

    /// Mark that an agent is waiting on another agent (for delegation)
    pub async fn set_waiting(&self, slot_id: &str, waiting_on_agent: &str) {
        let mut slots = self.active_slots.write().await;
        if let Some(slot) = slots.iter_mut().find(|s| s.slot_id == slot_id) {
            slot.waiting_on = Some(waiting_on_agent.to_string());
        }

        let mut graph = self.wait_graph.write().await;
        graph.insert(slot_id.to_string(), waiting_on_agent.to_string());
    }

    /// Clear waiting status
    pub async fn clear_waiting(&self, slot_id: &str) {
        let mut slots = self.active_slots.write().await;
        if let Some(slot) = slots.iter_mut().find(|s| s.slot_id == slot_id) {
            slot.waiting_on = None;
        }

        let mut graph = self.wait_graph.write().await;
        graph.remove(slot_id);
    }

    /// Detect deadlocks in the wait graph
    /// Returns cycles as Vec<Vec<String>> (each cycle is a list of agent IDs)
    pub async fn detect_deadlocks(&self) -> Vec<Vec<String>> {
        let slots = self.active_slots.read().await;
        let mut cycles = Vec::new();

        // Build agent -> waiting_on_agent map
        let mut wait_map: HashMap<String, String> = HashMap::new();
        for slot in slots.iter() {
            if let Some(ref waiting_on) = slot.waiting_on {
                wait_map.insert(slot.agent_id.clone(), waiting_on.clone());
            }
        }

        // DFS cycle detection
        let mut visited = std::collections::HashSet::new();
        for start in wait_map.keys() {
            if visited.contains(start) {
                continue;
            }

            let mut path = vec![start.clone()];
            let mut current = start.clone();
            let mut path_set = std::collections::HashSet::new();
            path_set.insert(start.clone());

            while let Some(next) = wait_map.get(&current) {
                if path_set.contains(next) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|p| p == next).unwrap();
                    cycles.push(path[cycle_start..].to_vec());
                    break;
                }
                if visited.contains(next) {
                    break;
                }
                path.push(next.clone());
                path_set.insert(next.clone());
                current = next.clone();
            }

            for p in &path {
                visited.insert(p.clone());
            }
        }

        cycles
    }

    /// Get current state for monitoring
    pub async fn get_status(&self) -> SchedulerStatus {
        let slots = self.active_slots.read().await;
        let mut lane_usage = HashMap::new();

        for (lane, config) in &self.lane_configs {
            let active = slots.iter().filter(|s| s.lane == *lane).count();
            lane_usage.insert(*lane, (active, config.max_concurrent));
        }

        drop(slots); // Release lock before calling detect_deadlocks
        let deadlocks = self.detect_deadlocks().await;

        SchedulerStatus {
            lane_usage,
            active_slots: self.active_slots.read().await.clone(),
            deadlocks,
        }
    }

    /// Get active slot count for a lane
    pub async fn lane_count(&self, lane: Lane) -> usize {
        let slots = self.active_slots.read().await;
        slots.iter().filter(|s| s.lane == lane).count()
    }
}

/// Scheduler status for monitoring
#[derive(Debug)]
pub struct SchedulerStatus {
    pub lane_usage: HashMap<Lane, (usize, usize)>, // (active, max)
    pub active_slots: Vec<ExecutionSlot>,
    pub deadlocks: Vec<Vec<String>>,
}
