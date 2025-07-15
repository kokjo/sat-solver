use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Design {
    pub creator: String,
    pub modules: BTreeMap<String, Module>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub attributes: serde_json::Value,
    pub ports: BTreeMap<String, Port>,
    pub cells: BTreeMap<String, Cell>,
    #[serde(rename="netnames")]
    pub nets: BTreeMap<String, Net>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "output")]
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub direction: Direction,
    pub bits: Vec<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    #[serde(rename = "type")]
    pub module: String,
    pub attributes: BTreeMap<String, serde_json::Value>,
    pub parameters: BTreeMap<String, serde_json::Value>,
    pub port_directions: BTreeMap<String, Direction>,
    pub connections: BTreeMap<String, Vec<u64>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Net {
    pub attributes: BTreeMap<String, serde_json::Value>,
    pub bits: Vec<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}