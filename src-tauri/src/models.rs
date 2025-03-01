use serde::{Deserialize, Serialize};
use shared::{Algorithm, Model};
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};

// from files
#[derive(Serialize, Deserialize, Clone)]
pub struct FileModel {
    pub name: String,
    pub children: Vec<String>,
    pub algorithm: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileTreeModel {
    pub root_name: String,
    pub data: Vec<FileModel>,
}

pub struct TreeModel {
    pub models: BTreeMap<u64, Model>,
    pub root_name: String,
    pub counter: AtomicU64,
}

pub type FileData = HashMap<String, f64>;
pub type Data = HashMap<u64, f64>;

#[derive(Default)]
pub struct TauriState {
    pub curr_tree_model: Option<TreeModel>,
    pub curr_file_path: Option<String>,
}
