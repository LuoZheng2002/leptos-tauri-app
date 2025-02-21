use serde::{Deserialize, Serialize};
use shared::{Algorithm, Model};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

// from files
#[derive(Serialize, Deserialize, Clone)]
pub struct FileModel {
    pub name: String,
    pub children: Vec<String>,
    pub algorithm: String,
}

pub struct TreeModel {
    pub models: HashMap<u64, Model>,
    pub root_name: String,
    pub counter: AtomicU64,
}
#[derive(Default)]
pub struct TauriState {
    pub curr_tree_model: Option<TreeModel>,
    pub curr_file_path: Option<String>,
}
