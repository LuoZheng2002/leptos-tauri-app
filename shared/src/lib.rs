// the frontend model is a hashmap of unique_id: model (id, name, ref_count, children_names, algorithm)
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Serialize, Deserialize, Clone, Debug, Default, EnumString, Display)]
pub enum Algorithm {
    #[default]
    //#[strum(serialize = "请选择/缺失/错误")]
    #[strum(serialize = "qingxuanze")]
    None,
    // #[strum(serialize = "求和")]
    #[strum(serialize = "qiuhe")]
    Sum,
    // #[strum(serialize = "取乘积")]
    #[strum(serialize = "quchengji")]
    Product,
    // #[strum(serialize = "取平均")]
    #[strum(serialize = "qupingjun")]
    Average,
    // #[strum(serialize = "取最大值")]
    #[strum(serialize = "quzuidazhi")]
    Max,
    // #[strum(serialize = "取最小值")]
    #[strum(serialize = "quzuixiaozhi")]
    Min,
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
    pub newName: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RenameResponse {
    RemoveSelfUpdateRelated {
        id_to_remove: u64,
        ids_to_update: Vec<u64>,
    },
    RenameSelf(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeleteResponse {
    pub id_to_remove: u64,
    pub ids_to_update: Vec<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct UpdateAlgorithmArgs {
    pub id: u64,
    pub newAlgorithm: Algorithm,
}
