use crate::app::{invoke, terminal_log};
use futures::future::Either;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::{ArcRwSignal, Get, GetUntracked, Set};
use leptos::task::spawn_local;
use serde_wasm_bindgen::{from_value, to_value};
use shared::{Algorithm, ExpandInfo, IdArgs, Model, MyResult};
use std::{collections::HashMap, future::Future};

#[derive(Clone, Debug, Default)]
pub struct ExpandSignal {
    pub algorithm: ArcRwSignal<Algorithm>,
    pub children: ArcRwSignal<Vec<u64>>,
}

#[derive(Clone, Debug, Default)]
pub struct TreeNodeModel {
    pub id: u64,
    pub name: ArcRwSignal<String>,
    pub ref_count: ArcRwSignal<u64>,
    pub expand_signal: ArcRwSignal<Option<ExpandSignal>>,
    pub value: ArcRwSignal<Option<f64>>,
}

pub struct LeptosContext {
    pub models: HashMap<u64, TreeNodeModel>,
    pub err_msg: ArcRwSignal<String>,
}

impl LeptosContext {
    // since locking the context is an asynchrnous operation,
    // let's just make get_model async anyway
    pub async fn get_model(&mut self, id: u64) -> TreeNodeModel {
        if let Some(model) = self.models.get(&id) {
            model.clone()
        } else {
            let tree_node_model = TreeNodeModel {
                id,
                name: ArcRwSignal::new("加载中".to_string()),
                ref_count: ArcRwSignal::new(0),
                expand_signal: ArcRwSignal::new(None),
                value: ArcRwSignal::new(None),
            };
            self.models.insert(id, tree_node_model);
            self.update_model(id).await;
            self.models.get(&id).unwrap().clone()
        }
    }
    pub async fn update_model(&mut self, id: u64) {
        console_log(&format!("update_model called with id: {}", id));
        let model = self.models.get(&id);
        if model.is_none() {
            console_log(&format!("Model {} does not exist in the front end", id));
        }
        let model = model.unwrap();
        let id_args = IdArgs { id };
        let id_args = to_value(&id_args).unwrap();
        let result = invoke("query_node", id_args).await;
        let result = from_value::<MyResult<Model, String>>(result).unwrap();
        match result {
            MyResult::Ok(new_model) => {
                model.name.set(new_model.name);
                model.ref_count.set(new_model.ref_count);
                // when the expand signal goes from none to some, create the signals
                // when the expand signal goes from some to none, delete the signals
                // when the expand signal goes from some to some, update the signals
                match (model.expand_signal.get_untracked(), new_model.expand_info) {
                    (Some(expand_signal), Some(new_expand_signal)) => {
                        expand_signal.algorithm.set(new_expand_signal.algorithm);
                        expand_signal.children.set(new_expand_signal.children);
                    }
                    (Some(_expand_signal), None) => {
                        model.expand_signal.set(None);
                    }
                    (None, Some(expand_signal)) => {
                        let new_expand_signal = ExpandSignal {
                            algorithm: ArcRwSignal::new(expand_signal.algorithm),
                            children: ArcRwSignal::new(expand_signal.children),
                        };
                        model.expand_signal.set(Some(new_expand_signal));
                    }
                    _ => {}
                }
                model.value.set(new_model.value);
            }
            MyResult::Err(e) => {
                // handle error
                terminal_log(&e).await;
            }
        }
    }
    pub fn update_values(&mut self, values: &HashMap<u64, f64>) {
        for (id, value) in values {
            if let Some(model) = self.models.get_mut(id) {
                model.value.set(Some(*value));
            }
        }
    }
}
