use core::error;
use std::sync::{Arc, RwLock};

use crate::models::LeptosContext;
use crate::pages::home::Home;
use crate::pages::save::Save;
use crate::pages::tree::Tree;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::hooks::use_navigate;
use leptos_router::path;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use shared::LogArgs;
use tokio::sync::Mutex;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}
pub async fn terminal_log(message: &str) {
    invoke(
        "log",
        to_value(&LogArgs {
            message: message.to_string(),
        })
        .unwrap(),
    )
    .await;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[component]
pub fn App() -> impl IntoView {
    let err_msg = ArcRwSignal::new(String::new());
    let leptos_context = Arc::new(Mutex::new(LeptosContext {
        models: Default::default(),
        err_msg: err_msg.clone(),
    }));
    provide_context(leptos_context);
    view! {
        <div>
            <Router>
                <Routes fallback=|| "Not found.">
                    // our root route: the contact list is always shown
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/tree") view=Tree />
                    <Route path=path!("/save") view=Save />
                    <Route path=path!("/*any") view=|| view! { <h1>"Not Found"</h1> } />
                </Routes>
            </Router>
            <div class="text-red-500 font-medium">{err_msg}</div>
        </div>
    }
}
