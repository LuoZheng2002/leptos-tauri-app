use crate::app::invoke;
use crate::components::tree_node::TreeNode;
use crate::models::{LeptosContext, TreeNodeModel};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use send_wrapper::SendWrapper;
use serde_wasm_bindgen::from_value;
use shared::{Algorithm, ExpandInfo, Model, MyResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use wasm_bindgen::JsValue;

#[component]
pub fn Tree() -> impl IntoView {
    let leptos_context = use_context::<Arc<Mutex<LeptosContext>>>().unwrap();
    let curr_file_path_data = LocalResource::new({
        let leptos_context = leptos_context.clone();
        move || {
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
        }
    });
    let curr_file_path = move || {
        curr_file_path_data
            .get()
            .as_deref()
            .map_or_else(|| "加载中".to_string(), |s| s.clone())
    };
    let navigate = use_navigate();
    let on_save = {
        let leptos_context = leptos_context.clone();
        move |_| {
            let leptos_context = leptos_context.clone();
            spawn_local(async move {
                let context = leptos_context.lock().await;
                let result = invoke("request_save", JsValue::NULL).await;
                let result = from_value::<MyResult<(), String>>(result).unwrap();
                match result {
                    MyResult::Ok(_) => {
                        context.err_msg.set("保存成功".to_string());
                    }
                    MyResult::Err(e) => {
                        context.err_msg.set(format!("{}", e));
                    }
                }
            });
        }
    };
    let on_back = {
        let leptos_context = leptos_context.clone();
        move |_| {
            let leptos_context = leptos_context.clone();
            let navigate = navigate.clone();
            spawn_local(async move {
                let mut context = leptos_context.lock().await;
                context.models.clear();
                context.err_msg.set("".to_string());
                navigate("/", Default::default());
            });
        }
    };

    let root_resource = LocalResource::new({
        let leptos_context = leptos_context.clone();
        move || {
            let leptos_context = leptos_context.clone();
            async move {
                let mut context = leptos_context.lock().await;
                context.get_model(0).await
            }
        }
    });

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
            <Suspense>
                {move || {
                    let model = root_resource.get();
                    model.map(|model|{
                        let model = model.into_taken();
                        view! {
                            <TreeNode tree_node_model=model/>
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
