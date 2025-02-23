use crate::helper::{suggest_new_name_add, suggest_new_name_dupe};
use crate::loader::load_models;
use crate::models::{TauriState, TreeModel};
use shared::{Algorithm, DeleteResponse, Model, MyResult, RenameResponse};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicU64, RwLock};
use tauri::AppHandle;
use tauri_plugin_dialog::{Dialog, DialogExt, FileDialogBuilder, FilePath};

fn select_file_helper(app: AppHandle) -> Result<String, String> {
    println!("select_file called");
    let file_path = app.dialog().file().blocking_pick_file();
    file_path
        .map(|path| match path {
            FilePath::Path(pathbuf) => pathbuf.to_string_lossy().to_string(),
            FilePath::Url(url) => url.to_string(),
        })
        .ok_or("未选择文件".to_string())
}

#[tauri::command]
pub fn select_file(app: AppHandle) -> MyResult<String, String> {
    match select_file_helper(app) {
        Ok(file_path) => MyResult::Ok(file_path),
        Err(e) => MyResult::Err(e),
    }
}

fn prepare_models_helper(
    file_path: &str,
    state: tauri::State<RwLock<TauriState>>,
) -> Result<(), String> {
    println!("prepare_models called");
    let mut state = state.write().unwrap();

    let tree_model = load_models(file_path)?;
    state.curr_file_path = Some(file_path.to_string());
    state.curr_tree_model = Some(tree_model);
    Ok(())
}

#[tauri::command]
pub fn prepare_models(
    file_path: &str,
    state: tauri::State<RwLock<TauriState>>,
) -> MyResult<(), String> {
    let result = prepare_models_helper(file_path, state);
    match result {
        Ok(_) => MyResult::Ok(()),
        Err(e) => MyResult::Err(e),
    }
}

fn query_file_path_helper(state: tauri::State<RwLock<TauriState>>) -> Result<String, String> {
    println!("query_file_path called");
    let state = state.read().unwrap();
    state
        .curr_file_path
        .as_ref()
        .cloned()
        .ok_or("文件路径加载错误".to_string())
}

#[tauri::command]
pub fn query_file_path(state: tauri::State<RwLock<TauriState>>) -> MyResult<String, String> {
    let result = query_file_path_helper(state);
    match result {
        Ok(file_path) => MyResult::Ok(file_path),
        Err(e) => MyResult::Err(e),
    }
}

fn query_node_helper(id: u64, state: tauri::State<RwLock<TauriState>>) -> Result<Model, String> {
    println!("Rust: query_node called with id: {}", id);
    let state = state.read().unwrap();
    let models = state
        .curr_tree_model
        .as_ref()
        .ok_or("模型未加载".to_string())?;
    models
        .models
        .get(&id)
        .cloned()
        .ok_or(format!("请求节点错误：未找到模型{}", id))
}

#[tauri::command]
pub fn query_node(id: u64, state: tauri::State<RwLock<TauriState>>) -> MyResult<Model, String> {
    let result = query_node_helper(id, state);
    match result {
        Ok(model) => MyResult::Ok(model),
        Err(e) => MyResult::Err(e),
    }
}

fn update_reference_count(models: &mut HashMap<u64, Model>) {
    let mut reference_counts = models
        .iter()
        .map(|(id, _)| (*id, 0))
        .collect::<HashMap<u64, u64>>();
    for (_, model) in models.iter() {
        if let Some(expand_info) = &model.expand_info {
            for child_id in expand_info.children.iter() {
                *reference_counts.get_mut(child_id).unwrap() += 1;
            }
        }
    }
    for (id, reference_count) in reference_counts.iter() {
        models.get_mut(id).unwrap().ref_count = *reference_count;
    }
}

fn delete_node_and_update_children(
    id: u64,
    models: &mut HashMap<u64, Model>,
) -> Result<HashSet<u64>, String> {
    models.remove(&id).ok_or(format!("未找到要删除的模型：{}", id))?;
    let mut ids_to_update = HashSet::new();
    for (_, model) in models.iter_mut() {
        if let Some(expand_info) = model.expand_info.as_mut() {
            if expand_info.children.contains(&id) {
                expand_info.children.retain(|child_id| *child_id != id);
                ids_to_update.insert(model.id);
            }
        }
    }
    update_reference_count(models);
    Ok(ids_to_update)
}

fn request_rename_helper(
    id: u64,
    new_name: &str,
    state: tauri::State<RwLock<TauriState>>,
) -> Result<RenameResponse, String> {
    // 1. If has children, and name is duplicated, return error
    // 2. If has no children, and name is duplicated, delete current model and replace other models' children -> (delete message, parent update messages)
    // 3. If name is not duplicated, rename -> (rename message)
    println!(
        "Rust: request_rename called with id: {}, new_name: {}",
        id, new_name
    );
    let mut state = state.write().unwrap();
    let models = &mut state
        .curr_tree_model
        .as_mut()
        .ok_or("模型未加载".to_string())?
        .models;
    let new_name_owner = models.iter().find(|(_, model)| model.name == new_name);
    let new_name_owner_id = new_name_owner.map(|(id, _)| *id);
    let model = models.get_mut(&id).ok_or(format!("未找到模型{}", id))?;
    if model.name == new_name {
        Err("重命名失败：新名称与旧名称相同".to_string())?;
    }
    if let Some(new_name_owner_id) = new_name_owner_id {
        if model.expand_info.is_some() {
            Err("重命名失败：新名称已存在".to_string())?;
        }
        let ids_to_update = delete_node_and_update_children(new_name_owner_id, models)?;
        Ok(RenameResponse::RemoveSelfUpdateRelated {
            id_to_remove: id,
            ids_to_update: ids_to_update.into_iter().collect(),
        })
    } else {
        model.name = new_name.to_string();
        Ok(RenameResponse::RenameSelf(new_name.to_string()))
    }
}

#[tauri::command]
pub fn request_rename(
    id: u64,
    new_name: &str,
    state: tauri::State<RwLock<TauriState>>,
) -> MyResult<RenameResponse, String> {
    let result = request_rename_helper(id, new_name, state);
    match result {
        Ok(response) => MyResult::Ok(response),
        Err(e) => MyResult::Err(e),
    }
}

fn request_delete_helper(id: u64, state: tauri::State<RwLock<TauriState>>) -> Result<DeleteResponse, String> {
    println!("Rust: request_delete called with id: {}", id);
    if id == 0{
        Err("根节点不可删除".to_string())?;
    }
    let mut state = state.write().unwrap();
    let models = &mut state
        .curr_tree_model
        .as_mut()
        .ok_or("模型未加载".to_string())?
        .models;
    let ids_to_update = delete_node_and_update_children(id, models)?;
    Ok(DeleteResponse {
        id_to_remove: id,
        ids_to_update: ids_to_update.into_iter().collect(),
    })
}

#[tauri::command]
pub fn request_delete(id: u64, state: tauri::State<RwLock<TauriState>>) -> MyResult<DeleteResponse, String> {
    let result = request_delete_helper(id, state);
    match result {
        Ok(id) => MyResult::Ok(id),
        Err(e) => MyResult::Err(e),
    }
}

fn request_add_helper(
    id: u64,
    state: tauri::State<RwLock<TauriState>>,
) -> Result<u64, String> {
    let mut state = state.write().unwrap();
    let tree_model = state
        .curr_tree_model
        .as_mut()
        .ok_or("添加错误：模型未加载".to_string())?;
    let model = tree_model
        .models
        .get_mut(&id)
        .ok_or(format!("添加错误：未找到模型{}", id))?;
    let expand_info = model.expand_info.as_mut().ok_or("添加失败：模型无子节点".to_string())?;
    let new_id = tree_model.counter.fetch_add(1, Ordering::Relaxed);
    expand_info.children.push(new_id);
    let new_name = suggest_new_name_add(&tree_model.models);
    let new_model = Model {
        id: new_id,
        name: new_name,
        ref_count: 1, // the only parent is the current model
        expand_info: None,
        value: None,
    };
    tree_model.models.insert(new_id, new_model);
    Ok(id)
}

#[tauri::command]
pub fn request_add(
    id: u64,
    state: tauri::State<RwLock<TauriState>>,
) -> MyResult<u64, String> {
    let result = request_add_helper(id, state);
    match result {
        Ok(new_id) => MyResult::Ok(new_id),
        Err(e) => MyResult::Err(e),
    }
}

fn request_update_algorithm_helper(
    id: u64,
    new_algorithm: Algorithm,
    state: tauri::State<RwLock<TauriState>>,
) -> Result<u64, String> {
    println!(
        "Rust: request_update_algorithm called with id: {}, algorithm: {:?}",
        id, new_algorithm
    );
    let mut state = state.write().unwrap();
    let models = state
        .curr_tree_model
        .as_mut()
        .ok_or("模型未加载".to_string())?;
    let model = models
        .models
        .get_mut(&id)
        .ok_or(format!("未找到模型{}", id))?;
    if let Some(expand_info) = model.expand_info.as_mut() {
        expand_info.algorithm = new_algorithm;
    } else {
        Err("更新算法失败：模型无子节点".to_string())?;
    }
    Ok(id)
}

#[tauri::command]
pub fn request_update_algorithm(
    id: u64,
    new_algorithm: Algorithm,
    state: tauri::State<RwLock<TauriState>>,
) -> MyResult<u64, String> {
    let result = request_update_algorithm_helper(id, new_algorithm, state);
    match result {
        Ok(id) => MyResult::Ok(id),
        Err(e) => MyResult::Err(e),
    }
}

fn request_can_expand_toggling_helper(
    id: u64,
    state: tauri::State<RwLock<TauriState>>,
) -> Result<(), String> {
    println!("Rust: request_can_expand_toggling called with id: {}", id);
    Ok(())
}

#[tauri::command]
pub fn request_can_expand_toggling(
    id: u64,
    state: tauri::State<RwLock<TauriState>>,
) -> MyResult<(), String> {
    let result = request_can_expand_toggling_helper(id, state);
    match result {
        Ok(_) => MyResult::Ok(()),
        Err(e) => MyResult::Err(e),
    }
}
// #[tauri::command]
// pub fn update_node_name(name: &str, new_name: &str, state: tauri::State<RwLock<TauriState>>) -> String {
//     todo!()
// }

// #[tauri::command]
// fn add_node(parent_name: &str, state: tauri::State<RwLock<TauriState>>) -> String {

//     let mut state = state.write().unwrap();
//     println!("Rust: add_node called with parent_name: {}", parent_name);
//     let models = &mut state.curr_tree_model.as_mut().unwrap_or_else(||{
//         eprintln!("错误：当前树模型不存在");
//         exit(1);
//     }).models;
//     let new_name = suggest_new_name_add(models);
//     // add_node_to_parent(parent_name, &new_name, &mut state.models);
//     new_name
// }

// #[tauri::command]
// fn delete_node(parent_name: &str, name: &str, state: tauri::State<RwLock<TauriState>>) {
//     let mut state = state.write().unwrap();
//     println!("delete_node called with name: {}", name);
//     // this is tricky because we should only delete the node inside its parent. If it is referenced by other nodes, we should not remove it entirely from the models
//     // if its reference count is 1, then we can remove it entirely
//     // if its reference count is more than 1, then we should only remove it from its parent
//     // remove_node_from_parent(parent_name, name, &mut state.models);
//     todo!()
// }

// #[tauri::command]
// fn query_root_name(state: tauri::State<RwLock<TauriState>>) -> String {
//     let state = state.read().unwrap();
//     state.curr_tree_model.as_ref().unwrap_or_else(||{
//         eprintln!("错误：当前树模型不存在");
//         exit(1);
//     }).root_name.clone()
// }

// #[tauri::command]
// fn query_children(parent_name: &str, state: tauri::State<Mutex<TauriState>>) -> Vec<String> {
//     println!("Rust: query_children called with parent_name: {}", parent_name);
//     let state = state.lock().unwrap();
//     let model = match state.models.get(parent_name) {
//         Some(model) => model,
//         None => {
//             eprintln!("query children 错误：未找到模型{}", parent_name);
//             exit(1);
//         }
//     };
//     match &model.children {
//         Some(children) => children.clone(),
//         None => {
//             eprintln!("错误：模型{}无子节点", parent_name);
//             exit(1);
//         }
//     }
// }
// #[tauri::command]
// fn query_algorithm(parent_name: &str, state: tauri::State<Mutex<TauriState>>) -> String {
//     println!("Rust: query_algorithm called with parent_name: {}", parent_name);
//     let state = state.lock().unwrap();
//     let model = match state.models.get(parent_name) {
//         Some(model) => model,
//         None => {
//             eprintln!("query algorithm 错误：未找到模型{}", parent_name);
//             exit(1);
//         }
//     };
//     match &model.algorithm {
//         Some(algorithm) => algorithm.clone(),
//         None => {
//             eprintln!("错误：模型{}无算法", parent_name);
//             exit(1);
//         }
//     }
// }

// #[tauri::command]
// fn query_ref_count(name: &str, state: tauri::State<Mutex<TauriState>>) -> u64 {
//     println!("Rust: query_ref_count called with name: {}", name);
//     let state = state.lock().unwrap();
//     match state.models.get(name) {
//         Some(model) => model.ref_count,
//         None => {
//             eprintln!("ref count 警告：可能被丢弃的模型{}", name);
//             0
//         }
//     }
// }
// #[tauri::command]
// fn toggle_has_children(name: &str, state: tauri::State<Mutex<TauriState>>) {
//     let mut state = state.lock().unwrap();
//     let model = state.models.get_mut(name).expect(format!("model {} does not exist", name).as_str());
//     match model.children{
//         Some(_)=>{
//             assert!(model.algorithm.is_some());
//             model.children = None;
//             model.algorithm = None;
//         }
//         None=>{
//             model.children = Some(vec![]);
//             model.algorithm = Some("未定义算法".to_string());
//         }
//     }
// }
// #[tauri::command]
// fn update_algorithm(name: &str, algorithm: &str, state: tauri::State<Mutex<TauriState>>) {
//     let mut state = state.lock().unwrap();
//     let model = state.models.get_mut(name).expect(format!("model {} does not exist", name).as_str());
//     model.algorithm = Some(algorithm.to_string());
// }

#[tauri::command]
pub fn log(message: &str) {
    println!("{}", message);
}
