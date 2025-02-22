use crate::components::tree_node_children::TreeNodeChildren;
use crate::{
    app::{invoke, terminal_log},
    models::LeptosContext,
};
use leptos::{
    either::Either,
    ev::{Event, KeyboardEvent, MouseEvent},
    leptos_dom::logging::console_log,
    prelude::*,
    task::spawn_local,
};
use leptos_icons::Icon;
use serde::de::value;
use serde_wasm_bindgen::{from_value, to_value};
use shared::{IdArgs, MyResult, RenameArgs, RenameResponse};
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;

#[component]
pub fn TreeNode(id: u64) -> impl IntoView {
    let leptos_context1 = use_context::<Arc<Mutex<LeptosContext>>>().unwrap();
    let leptos_context2 = leptos_context1.clone();
    let leptos_context3 = leptos_context1.clone();
    
    let (expanded, set_expanded) = signal(false);
    let (editing, set_editing) = signal(false);
    let (new_name, set_new_name) = signal(String::new());
    let tree_node_model = LocalResource::new(move || {
        let context = leptos_context1.clone();
        async move {
            console_log("Acquire lock in TreeNode");
            let mut context = context.lock().await;
            let model = context.get_model(id).await;
            console_log("Release lock in TreeNode");
            model
        }
    });
    let name = move || {
        tree_node_model
            .get()
            .as_deref()
            .map(|model| model.name.get())
            .unwrap_or("加载中".to_string())
    };
    let ref_count = move || {
        tree_node_model
            .get()
            .as_deref()
            .map(|model| model.ref_count.get())
            .unwrap_or(114514)
    };
    let expand_info = move || {
        tree_node_model
            .get()
            .as_deref()
            .map(|model| model.expand_info.get())
            .unwrap_or_default()
    };
    let value = move || {
        tree_node_model
            .get()
            .as_deref()
            .map(|model| model.value.get())
            .unwrap_or_default()
    };
    let on_rename1 = move || {
        set_editing.set(false);
        let new_name = new_name.get();
        let rename_args = RenameArgs {id, newName: new_name };
        let rename_args = to_value(&rename_args).unwrap();
        let leptos_context = leptos_context2.clone();
        spawn_local(async move {
            let result = invoke("request_rename", rename_args).await;
            let result = from_value::<MyResult<RenameResponse, String>>(result).unwrap();
            match result {
                MyResult::Ok(response) => {
                    let mut context = leptos_context.lock().await;
                    match response {
                        RenameResponse::RemoveSelfUpdateRelated {
                            id_to_remove,
                            ids_to_update,
                        } => {
                            console_log(&format!("remove id: {}", id_to_remove));
                            context.models.remove(&id_to_remove);
                            for parent in ids_to_update {
                                if context.models.contains_key(&parent) {
                                    context.update_model(parent).await;
                                }
                            }
                        },
                        RenameResponse::RenameSelf(new_name) => {
                            console_log(&format!("rename id: {}", id));
                            context.update_model(id).await;
                        }
                    }
                }
                MyResult::Err(e) => {
                    let context = leptos_context.lock().await;
                    context.err_msg.set(e);
                }
            }
        });
    };
    let on_rename2 = on_rename1.clone();
    let on_blur = move |_| {
        on_rename1();
    };
    let on_enter_down = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" {
            on_rename2();
        }
    };
    let on_delete = move |_| {
        let id_args = IdArgs { id };
        let id_args = to_value(&id_args).unwrap();
        spawn_local(async move {
            let result = invoke("request_delete", id_args).await;
            // process result
            todo!()
        });
    };
    let has_children = move || expand_info().is_some();
    let toggle_expand = move |_| {
        set_expanded.set(!expanded.get());
    };
    let set_editing_true = move |_| {
        set_editing.set(true);
    };
    // let set_editing_false = move |_|{
    //     set_editing.set(false);
    // };
    let request_can_expand_toggling = move |_| {
        let id_args = IdArgs { id };
        let id_args = to_value(&id_args).unwrap();
        spawn_local(async move {
            let result = invoke("request_can_expand_toggling", id_args).await;
            // process result
            todo!()
        });
    };

    view! {
        // Node Header
        <div class="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md">
            // Expand/Collapse Button for Parent Nodes
            {move || {
                if has_children() {
                    Either::Left(
                        view! {
                            <div on:click=toggle_expand>
                                {move || {
                                    if expanded.get() {
                                        view! {
                                            <Icon width="16" height="16" icon=icondata::LuChevronDown />
                                        }
                                    } else {
                                        view! {
                                            <Icon
                                                width="16"
                                                height="16"
                                                icon=icondata::LuChevronRight
                                            />
                                        }
                                    }
                                }}
                            </div>
                        },
                    )
                } else {
                    Either::Right(view! { <div class="w-4 h-4 inline-block" /> })
                }
            }} // {/* Folder or File Icon */}
            {move || {
                if has_children() {
                    view! { <Icon width="16" height="16" icon=icondata::LuFolder /> }
                } else {
                    view! { <Icon width="16" height="16" icon=icondata::LuFile /> }
                }
            }} // {/* Editable Name */}
            {move || {
                if editing.get() {
                    let on_blur = on_blur.clone();
                    let on_enter_down = on_enter_down.clone();
                    Either::Left(
                        view! {
                            <input
                                type="text"
                                value=name
                                bind:value=(new_name, set_new_name)
                                on:blur=on_blur
                                on:keydown=on_enter_down
                                autofocus
                                class="border px-1 rounded"
                            />
                        },
                    )
                } else {
                    Either::Right(
                        view! {
                            <span on:dblclick=set_editing_true>{name}</span>
                            <span class="ml-3">"id: "{id}</span>
                        },
                    )
                }
            }} // {/* Delete Button */}

            <div class="ml-auto">
                <button
                    class="text-blue-500 hover:text-blue-700 mr-3"
                    on:click=request_can_expand_toggling
                >
                    {move || {
                        if has_children() {
                            view! { <Icon width="16" height="16" icon=icondata::LuMinus /> }
                        } else {
                            view! { <Icon width="16" height="16" icon=icondata::LuPlus /> }
                        }
                    }}
                </button>
                <div class="inline-block mr-3">"引用计数："{ref_count}</div>
                <button class="text-red-500 hover:text-red-700" on:click=on_delete>
                    <Icon width="16" height="16" icon=icondata::LuTrash />
                </button>
            </div>
        </div>
        <div class="pl-4 border-l border-gray-300">
            // {/* Children Nodes (if expanded) */}
            {move || {
                if has_children() && expanded.get() {
                    view! { <TreeNodeChildren id=id /> }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}
        </div>
    }
}
