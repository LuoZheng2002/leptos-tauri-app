use crate::app::invoke;
use crate::components::tree_node::TreeNode;
use crate::models::{LeptosContext, TreeNodeModel};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use serde_wasm_bindgen::from_value;
use shared::{Algorithm, ExpandInfo, Model, MyResult};
use leptos::task::spawn_local;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use wasm_bindgen::JsValue;

#[component]
pub fn Tree() -> impl IntoView {
    let leptos_context = use_context::<Arc<Mutex<LeptosContext>>>().unwrap();
    let leptos_context2 = leptos_context.clone();
    let curr_file_path_data = LocalResource::new(move || {
        let leptos_context = leptos_context.clone();
        async move {
            let result = invoke("query_file_path", JsValue::NULL).await;
            let result = from_value::<MyResult<String, String>>(result).unwrap();
            match result {
                MyResult::Ok(file_path) => file_path,
                MyResult::Err(e) => {
                    leptos_context
                        .lock()
                        .await
                        .err_msg
                        .set(format!("错误信息：{}", e));
                    "获取文件名错误".to_string()
                }
            }
        }
    });
    let curr_file_path = move || {
        curr_file_path_data
            .get()
            .as_deref()
            .map_or_else(|| "加载中".to_string(), |s| s.clone())
    };
    let navigate = use_navigate();
    let on_save = move |_| {};
    let on_back = move |_| {
        let leptos_context = leptos_context2.clone();
        let navigate = navigate.clone();
        spawn_local(async move {
            let mut leptos_context = leptos_context.lock().await;
            leptos_context.models.clear();
            navigate("/", Default::default());
        });
    };

    view! {
        <div class="p-4">
            <div class="inline-block">
                <button
                    on:click=on_save
                    class="mx-3 mb-2 px-4 py-2 bg-blue-600 text-white font-semibold rounded-2xl shadow-md hover:bg-blue-700 transition-all duration-200 ease-in-out active:scale-95"
                >
                    "保存"
                </button>
                <button
                    on:click=on_back
                    class="mx-3 px-4 py-2 bg-blue-600 text-white font-semibold rounded-2xl shadow-md hover:bg-blue-700 transition-all duration-200 ease-in-out active:scale-95"
                >
                    "返回"
                </button>
            </div>
            <h1 class="text-xl font-bold mb-4">"文件："{curr_file_path}</h1>
            <TreeNode id=0 />
        </div>
    }
}
