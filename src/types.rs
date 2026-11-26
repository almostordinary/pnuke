use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub port: u16,
    pub protocol: String,
    pub ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillResult {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub success: bool,
    pub message: String,
    pub duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectResult {
    pub port: u16,
    pub occupied: bool,
    pub processes: Vec<ProcessInfo>,
}
