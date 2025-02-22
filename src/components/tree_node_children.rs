use std::{str::FromStr, sync::{Arc, RwLock}};

use leptos::{either::Either, ev::Event, leptos_dom::logging::console_log, prelude::*, task::spawn_local};
use leptos_icons::Icon;
use serde_wasm_bindgen::{from_value, to_value};
use shared::{Algorithm, MyResult, UpdateAlgorithmArgs};
use tokio::sync::Mutex;

use crate::{app::{invoke, terminal_log}, components::tree_node::TreeNode, models::LeptosContext};

#[component]
pub fn TreeNodeChildren(id: u64) -> impl IntoView {
    let leptos_context = use_context::<Arc<Mutex<LeptosContext>>>().unwrap();
    let leptos_context2 = leptos_context.clone();
    let tree_node_model = LocalResource::new(move || {
        let context = leptos_context.clone();
        async move {
            console_log("Acquire lock in TreeNodeChildren");
            let mut context = context.lock().await;
            let model = context.get_model(id).await;
            console_log("Release lock in TreeNodeChildren");
            model
        }
    });
    let (editing, set_editing) = signal(false);
    let expand_info = move || {
        tree_node_model
            .get()
            .as_deref()
            .map(|model| model.expand_info.get())
            .unwrap_or_default()
    };
    let on_algorithm_change = move |ev: Event| {
        let algorithm_str = event_target_value(&ev);
        let algorithm = Algorithm::from_str(&algorithm_str).unwrap();
        console_log(&format!("form中的算法字符串：{}，算法枚举量：{:?}", algorithm_str, algorithm));
        let leptos_context = leptos_context2.clone();
        spawn_local(async move{
            let update_algorithm_args = UpdateAlgorithmArgs {
                id,
                newAlgorithm: algorithm,
            };
            let update_algorithm_args = to_value(&update_algorithm_args).unwrap();
            let response = invoke("request_update_algorithm", update_algorithm_args).await;
            let response = from_value::<MyResult<u64, String>>(response).unwrap();
            match response{
                MyResult::Ok(id) => {
                    let mut context = leptos_context.lock().await;
                    context.update_model(id).await;
                }
                MyResult::Err(e) => {
                    terminal_log(&format!("更新算法失败：{}", e)).await;
                }
            }
        });
    };
    let children = move || {
        expand_info()
            .map(|info| info.children.clone())
            .unwrap_or_default()
    };
    // let set_editing_true = move |_| set_editing.set(true);
    let algorithm = move || {
        let algorithm = expand_info()
            .map(|info| info.algorithm.clone())
            .unwrap_or(Algorithm::None);
        console_log(&format!("收到的算法：{:?}", algorithm));
        console_log(&format!("算法字符串：{}", algorithm.to_string()));
        algorithm.to_string()
    };
    let on_add = move |_| {};
    view! {
        {move || {
            let on_algorithm_change = on_algorithm_change.clone();
            if expand_info().is_none() {
                Either::Left(view! { <div>"加载中..."</div> })
            } else {
                Either::Right(
                    view! {
                        <div class="transition-opacity duration-500 ease-in-out opacity-100">
                            <div class="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md">
                                {move || {
                                let on_algorithm_change = on_algorithm_change.clone();
                                    // if editing.get() {
                                            view! {
                                                <div class="w-4 h-4 inline-block" />
                                                <Icon
                                                    width="16"
                                                    height="16"
                                                    icon=icondata::LuCircuitBoard
                                                />
                                                <div class="inline-block">"算法："</div>
                                                <select
                                                    class="inline-block border border-gray-300 rounded p-2"
                                                    on:change=on_algorithm_change
                                                    prop:value=algorithm
                                                >
                                                    <option value=Algorithm::None
                                                        .to_string()>{Algorithm::None.to_string()}</option>
                                                    <option value=Algorithm::Sum
                                                        .to_string()>{Algorithm::Sum.to_string()}</option>
                                                    <option value=Algorithm::Product
                                                        .to_string()>{Algorithm::Product.to_string()}</option>
                                                    <option value=Algorithm::Average
                                                        .to_string()>{Algorithm::Average.to_string()}</option>
                                                    <option value=Algorithm::Max
                                                        .to_string()>{Algorithm::Max.to_string()}</option>
                                                    <option value=Algorithm::Min
                                                        .to_string()>{Algorithm::Min.to_string()}</option>
                                                </select>
                                            }.into_any()
                                    // } else {
                                    //         view! { <div class="w-4 h-4 inline-block"></div> }.into_any()
                                    // }
                                }}
                            </div>
                            <For
                                each=move || children()
                                key=|child| *child
                                children=move |id| { view! { <TreeNode id=id /> }.into_any() }
                            />
                            <div class="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md">
                                <div class="w-4 h-4 inline-block" />
                                <button on:click=on_add class="text-blue-500 hover:text-blue-700">
                                    "添加"
                                </button>
                            </div>
                        </div>
                    },
                )
            }
        }}
    }
}
