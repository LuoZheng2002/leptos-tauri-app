use crate::components::tree_node_children::TreeNodeChildren;
use crate::models::TreeNodeModel;
use crate::{
    app::{invoke, terminal_log},
    models::LeptosContext,
};
use leptos::server_fn::response;
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
use shared::{DeleteResponse, IdArgs, MyResult, RenameArgs, RenameResponse};
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;

#[component]
pub fn TreeNode(tree_node_model: TreeNodeModel) -> impl IntoView {
    let leptos_context = use_context::<Arc<Mutex<LeptosContext>>>().unwrap();

    let TreeNodeModel {
        id,
        name,
        ref_count,
        expand_signal,
        value,
    } = tree_node_model;

    let (expanded, set_expanded) = signal(false);
    let (editing, set_editing) = signal(false);
    let (new_name, set_new_name) = signal(String::new());

    let on_rename = {
        let leptos_context = leptos_context.clone();
        let name = name.clone();
        move || {
            let leptos_context = leptos_context.clone();
            let name = name.clone();
            let new_name = new_name.clone();
            set_editing.set(false);
            spawn_local(async move {
                // lock before doing any operation
                let mut context = leptos_context.lock().await;
                // if this node is already deleted, exit
                if !context.models.contains_key(&id) {
                    return;
                }
                let new_name = new_name.get_untracked();
                if new_name == name.get_untracked() {
                    return;
                }
                let rename_args = RenameArgs {
                    id,
                    newName: new_name,
                };
                let rename_args = to_value(&rename_args).unwrap();
                let result = invoke("request_rename", rename_args).await;

                let result = from_value::<MyResult<RenameResponse, String>>(result).unwrap();
                match result {
                    MyResult::Ok(response) => match response {
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
                        }
                        RenameResponse::RenameSelf(new_name) => {
                            console_log(&format!("rename id: {}", id));
                            context.update_model(id).await;
                        }
                    },
                    MyResult::Err(e) => {
                        context.err_msg.set(e);
                    }
                }
            });
        }
    };
    let on_blur = {
        let on_rename = on_rename.clone();
        move |_| {
            on_rename();
        }
    };
    let on_enter_down = move |ev: KeyboardEvent| {
        if ev.key() == "Enter" {
            on_rename();
        }
    };
    let on_delete = {
        let leptos_context = leptos_context.clone();
        move |_| {
            let leptos_context = leptos_context.clone();
            spawn_local(async move {
                // lock before doing any operation
                let mut context = leptos_context.lock().await;
                // if this node is already deleted, exit
                if !context.models.contains_key(&id) {
                    return;
                }
                let id_args = IdArgs { id };
                let id_args = to_value(&id_args).unwrap();
                let response = invoke("request_delete", id_args).await;
                let response = from_value::<MyResult<DeleteResponse, String>>(response).unwrap();
                match response {
                    MyResult::Ok(DeleteResponse {
                        id_to_remove,
                        ids_to_update,
                    }) => {
                        context.models.remove(&id_to_remove);
                        for parent in ids_to_update {
                            if context.models.contains_key(&parent) {
                                context.update_model(parent).await;
                            }
                        }
                    }
                    MyResult::Err(e) => {
                        context.err_msg.set(e);
                    }
                }
            });
        }
    };
    let expand_signal2 = expand_signal.clone();

    let has_children = move || expand_signal.get().is_some();
    let has_children2 = has_children.clone();
    let has_children3 = has_children.clone();
    let has_children4 = has_children.clone();

    let toggle_expand = move |_| {
        set_expanded.set(!expanded.get());
    };
    let set_editing_true = move |_| {
        set_editing.set(true);
    };
    // let set_editing_false = move |_|{
    //     set_editing.set(false);
    // };
    let request_can_expand_toggling = {
        let leptos_context = leptos_context.clone();
        move |_| {
            let leptos_context = leptos_context.clone();
            spawn_local(async move {
                let mut context = leptos_context.lock().await;
                let id_args = IdArgs { id };
                let id_args = to_value(&id_args).unwrap();
                let response = invoke("request_can_expand_toggling", id_args).await;
                let response = from_value::<MyResult<u64, String>>(response).unwrap();
                match response {
                    MyResult::Ok(id) => {
                        context.update_model(id).await;
                    }
                    MyResult::Err(e) => {
                        context.err_msg.set(e);
                    }
                }
            });
        }
    };

    let on_change = move |ev| {
        set_new_name.set(event_target_value(&ev));
    };

    view! {
        // Node Header
        <div class="flex items-center gap-2 cursor-pointer hover:bg-gray-100 p-1 rounded-md">
            // Expand/Collapse Button for Parent Nodes
            {move || {
                // let has_children = has_children.clone();
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
                // let has_children = has_children.clone();
                if has_children2() {
                    view! { <Icon width="16" height="16" icon=icondata::LuFolder /> }
                } else {
                    view! { <Icon width="16" height="16" icon=icondata::LuFile /> }
                }
            }} // {/* Editable Name */}
            {move || {
                if editing.get() {
                    let on_blur = on_blur.clone();
                    let on_enter_down = on_enter_down.clone();
                    let name = name.clone();
                    Either::Left(
                        view! {
                            <input
                                type="text"
                                prop:value=move|| name.get()
                                on:change=on_change
                                on:blur=on_blur
                                on:keydown=on_enter_down
                                autofocus
                                class="border px-1 rounded"
                            />
                        },
                    )
                } else {
                    let name = name.clone();
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
                        if has_children3() {
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
                if has_children4() && expanded.get() {

                    view! { <TreeNodeChildren id=id expand_signal=expand_signal2.get().unwrap() /> }
                        .into_any()
                } else {
                    view! {}.into_any()
                }
            }}
        </div>
    }
}
