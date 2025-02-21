use std::result;
use std::sync::{Arc, RwLock};

use crate::app::{invoke, terminal_log};
use crate::models::LeptosContext;
use leptos::task::spawn_local;
use leptos::{ev::Event, prelude::*};
use leptos_router::hooks::use_navigate;
use serde_wasm_bindgen::{from_value, to_value};
use shared::{LogArgs, MyResult, PrepareModelArgs};
use wasm_bindgen::JsValue;

#[component]
pub fn Home() -> impl IntoView {
    let (file_path, set_file_path) = signal(String::new());
    let (root_name, set_root_name) = signal(String::new());
    let leptos_context1 = use_context::<Arc<RwLock<LeptosContext>>>().unwrap();
    let leptos_context2 = leptos_context1.clone();
    let navigate = use_navigate();
    // Function to open the file dialog and get the selected file path
    let open_file_dialog = move |_| {
        let leptos_context = leptos_context1.clone();
        spawn_local(async move {
            terminal_log("打开文件对话框").await;
            // Open the file dialog and get the file path
            let file_path_result = invoke("select_file", JsValue::NULL).await;
            let file_path_result =
                from_value::<MyResult<String, String>>(file_path_result).unwrap();
            match file_path_result {
                MyResult::Ok(path) => {
                    // Update the file_path signal with the selected file path
                    set_file_path.set(path);
                }
                MyResult::Err(e) => {
                    // Handle error if needed
                    leptos_context
                        .write()
                        .unwrap()
                        .err_msg
                        .set(format!("错误信息：{}", e));
                }
            }
        });
    };
    let submit = move |_| {
        let leptos_context = leptos_context2.clone();
        let prepare_model_args = PrepareModelArgs {
            filePath: file_path.get().to_string(),
            rootName: root_name.get().to_string(),
        };
        let navigate = navigate.clone();
        spawn_local(async move {
            terminal_log("提交").await;
            let result = invoke("prepare_models", to_value(&prepare_model_args.clone()).unwrap()).await;
            let result = from_value::<MyResult<(), String>>(result).unwrap();
            match result {
                MyResult::Ok(_) => {
                    navigate("/tree", Default::default());
                }
                MyResult::Err(e) => {
                    // Handle error
                    leptos_context
                        .write()
                        .unwrap()
                        .err_msg
                        .set(format!("错误信息：{}", e));
                }
            }
        });
    };

    view! {
        <div class="flex flex-col items-center p-6 space-y-4 bg-gray-100 rounded-lg shadow-md">
            <button
                class="px-4 py-2 text-white bg-blue-500 rounded-lg hover:bg-blue-600 transition"
                on:click=open_file_dialog
            >
                "选择文件"
            </button>

            <p class="text-gray-700 font-medium">
                "选择的文件路径：" <span class="text-blue-600">{file_path}</span>
            </p>
            <div class="flex items-center space-x-2">
                <div class="inline-block">"根节点名称："</div>
                <input
                    type="text"
                    bind:value=(root_name, set_root_name)
                    placeholder="Type something"
                    class="px-3 py-2 border rounded-lg shadow-sm focus:ring focus:ring-blue-300"
                />
            </div>

            <button
                class="px-4 py-2 text-white bg-green-500 rounded-lg hover:bg-green-600 transition"
                on:click=submit
            >
                "Submit"
            </button>
        </div>
    }
}
