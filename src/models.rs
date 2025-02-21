use crate::app::{invoke, terminal_log};
use futures::future::Either;
use leptos::prelude::{RwSignal, Set};
use serde_wasm_bindgen::{from_value, to_value};
use shared::{ExpandInfo, IdArgs, Model, MyResult};
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
        let id_args = IdArgs { id };
        let id_args = to_value(&id_args).unwrap();
        self.models
            .get(&id)
            .map_or_else(
                || {
                    Either::Left(async {
                        let result = invoke("query_node", id_args).await;
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
                                terminal_log(&e).await;
                                TreeNodeModel::default()
                            }
                        }
                    })
                },
                |model| Either::Right(async { model.clone() }),
            )
            .await
    }
    pub async fn update_model(&mut self, id: u64){
        let model = self.models.get(&id).expect("Update model is only called when the model exists in the frontend");
        let id_args = IdArgs { id };
        let id_args = to_value(&id_args).unwrap();
        let result = invoke("query_node", id_args).await;
        let result = from_value::<MyResult<Model, String>>(result).unwrap();
        match result {
            MyResult::Ok(new_model) => {
                model.name.set(new_model.name);
                model.ref_count.set(new_model.ref_count);
                model.expand_info.set(new_model.expand_info);
                model.value.set(new_model.value);
            }
            MyResult::Err(e) => {
                // handle error
                terminal_log(&e).await;
            }
        }
    }
}
