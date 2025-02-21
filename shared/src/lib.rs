// the frontend model is a hashmap of unique_id: model (id, name, ref_count, children_names, algorithm)
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum Algorithm {
    #[default]
    None,
    Sum,
    Product,
    Average,
    Max,
    Min,
}
impl Algorithm {
    pub fn to_string(&self) -> String {
        match self {
            Algorithm::None => "请选择/错误/缺失".to_string(),
            Algorithm::Sum => "求和".to_string(),
            Algorithm::Product => "取乘积".to_string(),
            Algorithm::Average => "取平均".to_string(),
            Algorithm::Max => "取最大".to_string(),
            Algorithm::Min => "取最小".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExpandInfo {
    pub algorithm: Algorithm,
    pub children: Vec<u64>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Model {
    pub id: u64,
    pub name: String,
    pub ref_count: u64,
    pub expand_info: Option<ExpandInfo>,
    pub value: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct PrepareModelArgs {
    pub filePath: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogArgs {
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MyResult<T, E> {
    Ok(T),
    Err(E),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdArgs {
    pub id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct RenameArgs {
    pub id: u64,
    pub new_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RenameResponse {
    RemoveSelfUpdateParents {
        id_to_remove: u64,
        parents: Vec<u64>,
    },
    RenameSelf(String),
}
