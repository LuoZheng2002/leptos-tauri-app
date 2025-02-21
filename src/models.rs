use crate::app::invoke;
use futures::future::Either;
use leptos::prelude::RwSignal;
use serde_wasm_bindgen::{from_value, to_value};
use shared::{ExpandInfo, Model, MyResult, IdArgs};
use std::pin::pin;
use std::{collections::HashMap, future::Future};

#[derive(Clone, Debug, Default)]
pub struct TreeNodeModel {
    pub id: u64,
    pub name: RwSignal<String>,
    pub ref_count: RwSignal<u64>,
    pub expand_info: RwSignal<Option<ExpandInfo>>,
    pub value: RwSignal<Option<f64>>,
}

pub struct LeptosContext {
    pub models: HashMap<u64, TreeNodeModel>,
    pub err_msg: RwSignal<String>,
}

impl LeptosContext {
    pub async fn get_model(&mut self, id: u64) -> TreeNodeModel {
        let query_node_args = IdArgs { id };
        let query_node_args = to_value(&query_node_args).unwrap();
        self.models
            .get(&id)
            .map_or_else(
                || {
                    Either::Left(async {
                        let result = invoke("query_node", query_node_args).await;
                        let result = from_value::<MyResult<Model, String>>(result).unwrap();
                        match result {
                            MyResult::Ok(model) => TreeNodeModel {
                                id: model.id,
                                name: RwSignal::new(model.name),
                                ref_count: RwSignal::new(model.ref_count),
                                expand_info: RwSignal::new(model.expand_info),
                                value: RwSignal::new(model.value),
                            },
                            MyResult::Err(e) => {
                                // handle error
                                TreeNodeModel::default()
                            }
                        }
                    })
                },
                |model| Either::Right(async { model.clone() }),
            )
            .await
    }
}
