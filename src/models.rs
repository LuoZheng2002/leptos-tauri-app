use crate::app::{invoke, terminal_log};
use futures::future::Either;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::{ArcRwSignal, Set};
use serde_wasm_bindgen::{from_value, to_value};
use shared::{ExpandInfo, IdArgs, Model, MyResult};
use std::pin::pin;
use std::{collections::HashMap, future::Future};

#[derive(Clone, Debug, Default)]
pub struct TreeNodeModel {
    pub id: u64,
    pub name: ArcRwSignal<String>,
    pub ref_count: ArcRwSignal<u64>,
    pub expand_info: ArcRwSignal<Option<ExpandInfo>>,
    pub value: ArcRwSignal<Option<f64>>,
}

pub struct LeptosContext {
    pub models: HashMap<u64, TreeNodeModel>,
    pub err_msg: ArcRwSignal<String>,
}

impl LeptosContext {
    pub async fn get_model(&mut self, id: u64) -> TreeNodeModel {
        if !self.models.contains_key(&id)
        {
            let id_args = IdArgs { id };
            let id_args = to_value(&id_args).unwrap();
            let result = invoke("query_node", id_args).await;
            let result = from_value::<MyResult<Model, String>>(result).unwrap();
            let model = match result {
                MyResult::Ok(model) => TreeNodeModel {
                    id: model.id,
                    name: ArcRwSignal::new(model.name),
                    ref_count: ArcRwSignal::new(model.ref_count),
                    expand_info: ArcRwSignal::new(model.expand_info),
                    value: ArcRwSignal::new(model.value),
                },
                MyResult::Err(e) => {
                    // handle error
                    terminal_log(&e).await;
                    TreeNodeModel::default()
                }
            };
            self.models.insert(id, model);
        }
        self.models.get(&id).unwrap().clone()
    }
    pub async fn update_model(&mut self, id: u64){
        console_log(&format!("update_model called with id: {}", id));
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
