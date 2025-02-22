use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};

use futures::future::join_all;
use leptos::{
    either::Either, ev::Event, leptos_dom::logging::console_log, prelude::*, task::spawn_local,
};
use leptos_icons::Icon;
use serde_wasm_bindgen::{from_value, to_value};
use shared::{Algorithm, MyResult, UpdateAlgorithmArgs};
use tokio::sync::Mutex;

use crate::{
    app::{invoke, terminal_log},
    components::tree_node::TreeNode,
    models::{ExpandSignal, LeptosContext, TreeNodeModel},
};

// asynchronously retrieve the tree_node_model for each child, if every mode is ready, display the children
// ready is notified by a signal
// if the children in the expand_signal changes, ...


// when one of the ids of the children get modified, we do not want all children to be invalidated temporarily
// the children_ids will produce a signal, which should be used in an effect to load new children asynchronously
// we need a separate signal for the actual rendering

#[component]
pub fn TreeNodeChildren(id: u64, expand_signal: ExpandSignal) -> impl IntoView {
    let leptos_context = use_context::<Arc<Mutex<LeptosContext>>>().unwrap();
    let leptos_context2 = leptos_context.clone();

    let ExpandSignal {
        algorithm,
        children: children_ids,
    } = expand_signal;

    // let children_resource = LocalResource::new(||{
    //     async move {
    //         let children_ids = children_ids.get();
    //         let children = children_ids
    //             .iter()
    //             .map(|id| async {
    //                 let mut context = leptos_context.lock().await;
    //                 let model = context.get_model(*id).await;
    //                 (*id, model)
    //             })
    //             .collect::<Vec<_>>();
    //         // join all the futures
    //         join_all(children)
    //             .await
    //             .into_iter()
    //             .collect::<Vec<_>>()
    //         }
    // }
    // );

    let (children, set_children) = signal::<Vec<(usize, TreeNodeModel)>>(Default::default());

    Effect::new(move || {
        let children_ids = children_ids.get();
        let leptos_context = leptos_context.clone();
        spawn_local(async move {
            let children = children_ids
                .iter()
                .map(|id| async {
                    let mut context = leptos_context.lock().await;
                   context.get_model(*id).await
                })
                .collect::<Vec<_>>();
            // join all the futures
            let children = join_all(children).await
                .into_iter()
                .enumerate()
                .collect::<Vec<_>>();
            console_log(&format!("children count: {}", children.len()));
            set_children.set(children);
        });
    });

    let (editing, set_editing) = signal(false);

    let algorithm_blink = LocalResource::new(||{
        async{
            ()
        }
    });

    let on_algorithm_change = move |ev: Event| {
        let algorithm_str = event_target_value(&ev);
        let algorithm = Algorithm::from_str(&algorithm_str).unwrap();
        console_log(&format!(
            "form中的算法字符串：{}，算法枚举量：{:?}",
            algorithm_str, algorithm
        ));
        let leptos_context = leptos_context2.clone();
        spawn_local(async move {
            let update_algorithm_args = UpdateAlgorithmArgs {
                id,
                newAlgorithm: algorithm,
            };
            let update_algorithm_args = to_value(&update_algorithm_args).unwrap();
            let response = invoke("request_update_algorithm", update_algorithm_args).await;
            let response = from_value::<MyResult<u64, String>>(response).unwrap();
            match response {
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

    let on_add = move |_| {};
    view! {
        <div class="transition-opacity duration-500 ease-in-out opacity-100">
            <div class="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md">
                <div class="w-4 h-4 inline-block" />
                <Icon width="16" height="16" icon=icondata::LuCircuitBoard />
                <div class="inline-block">"算法："</div>
                <select
                    class="inline-block border border-gray-300 rounded p-2"
                    on:change=on_algorithm_change
                    prop:value=move ||{
                        if algorithm_blink.get().is_some(){
                        console_log(&format!("更新显示的算法：字符串：{}，枚举：{:?}", algorithm.get().to_string(), algorithm.get()));
                        algorithm.get().to_string()
                    }else{
                        console_log("算法栏初始化");
                        Algorithm::None.to_string()
                    }
                    }
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
                each=move||children.get()
                key=|(index, _model)| *index
                children=move |(_id, model)| {
                    view! { <TreeNode tree_node_model=model /> }.into_any()
                }
            />
            
            <div class="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md">
                <div class="w-4 h-4 inline-block" />
                <button on:click=on_add class="text-blue-500 hover:text-blue-700">
                    "添加"
                </button>
            </div>
        </div>
    }
}
