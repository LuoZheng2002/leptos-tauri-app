use std::collections::HashMap;
// the frontend model is a hashmap of unique_id: model (id, name, ref_count, children_names, algorithm)
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Serialize, Deserialize, Clone, Debug, Default, EnumString, Display)]
pub enum Algorithm {
    #[default]
    #[strum(serialize = "请选择/缺失/错误")]
    //#[strum(serialize = "qingxuanze")]
    None,
    #[strum(serialize = "求和")]
    // #[strum(serialize = "qiuhe")]
    Sum,
    #[strum(serialize = "取乘积")]
    // #[strum(serialize = "quchengji")]
    Product,
    #[strum(serialize = "取平均")]
    // #[strum(serialize = "qupingjun")]
    Average,
    #[strum(serialize = "取最大值")]
    // #[strum(serialize = "quzuidazhi")]
    Max,
    #[strum(serialize = "取最小值")]
    // #[strum(serialize = "quzuixiaozhi")]
    Min,
}

impl Algorithm {
    pub fn calculate(&self, data: &Vec<f64>) -> f64 {
        let result = match self {
            Algorithm::None => 0.0,
            Algorithm::Sum => data.iter().sum(),
            Algorithm::Product => data.iter().product(),
            Algorithm::Average => {
                if data.is_empty() {
                    0.0
                } else {
                    data.iter().sum::<f64>() / data.len() as f64
                }
            }
            Algorithm::Max => *data
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(&0.0),
            Algorithm::Min => *data
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(&0.0),
        };
        result
    }

    pub fn random(rand_num: f64)->Self{
        let size = 5;
        let index = ((rand_num * size as f64).floor() as usize).min(size - 1);
        let algorithm = match index {
            0 => Algorithm::Sum,
            1 => Algorithm::Product,
            2 => Algorithm::Average,
            3 => Algorithm::Max,
            4 => Algorithm::Min,
            _ => Algorithm::None,
        };
        algorithm
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
pub struct DeleteArgs {
    pub id: u64,
    pub parent: Option<u64>,
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
    pub id_to_remove: Option<u64>,
    pub ids_to_update: Vec<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct UpdateAlgorithmArgs {
    pub id: u64,
    pub newAlgorithm: Algorithm,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct QueryValuesArgs {
    pub ids: Vec<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueryValuesResponse {
    pub values: HashMap<String, f64>,
}
