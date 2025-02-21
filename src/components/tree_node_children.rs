use std::sync::{Arc, RwLock};

use leptos::{either::Either, leptos_dom::logging::console_log, prelude::*};
use leptos_icons::Icon;
use shared::Algorithm;
use tokio::sync::Mutex;

use crate::{app::terminal_log, components::tree_node::TreeNode, models::LeptosContext};

#[component]
pub fn TreeNodeChildren(id: u64) -> impl IntoView {
    console_log("TreeNodeChildren called");
    let leptos_context = use_context::<Arc<Mutex<LeptosContext>>>().unwrap();
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
    console_log("Alive");
    let expand_info = move || {
        tree_node_model
            .get()
            .as_deref()
            .map(|model| model.expand_info.get())
            .unwrap_or_default()
    };
    console_log("Alive");
    let on_algorithm_change = move |_|{

    };
    console_log("Alive");
    let children = move ||{
        expand_info()
            .map(|info| info.children.clone())
            .unwrap_or_default()
    };
    console_log("Alive");
    let on_add = move |_|{
    
    };
    console_log("Still Alive");
    view! {
        {move || {
            if expand_info().is_none() {
                Either::Left(view! { <div>"加载中..."</div> })
            } else {
                Either::Right(
                    view! {
                        <div class="transition-opacity duration-500 ease-in-out opacity-100">
                            <div class="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md">
                                <div class="w-4 h-4 inline-block" />
                                <Icon width="16" height="16" icon=icondata::LuCircuitBoard />
                                <div class="inline-block">"算法："</div>
                                <select
                                    class="inline-block border border-gray-300 rounded p-2"
                                    on:change=on_algorithm_change
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