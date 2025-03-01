use crate::helper::{suggest_new_name_add, suggest_new_name_dupe};
use crate::loader::{load_data, load_models};
use crate::models::{self, FileData, FileModel, FileTreeModel, TauriState, TreeModel};
use crate::saver::save_models;
use shared::{
    Algorithm, DeleteResponse, ExpandInfo, Model, MyResult, QueryValuesResponse, RenameResponse,
};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicU64, RwLock};
use tauri::AppHandle;
use tauri_plugin_dialog::{Dialog, DialogExt, FileDialogBuilder, FilePath, MessageDialogButtons};

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

fn update_reference_count(models: &mut BTreeMap<u64, Model>) {
    let mut reference_counts = models
        .iter()
        .map(|(id, _)| (*id, 0))
        .collect::<HashMap<u64, u64>>();
    println!("更新reference count");
    for (_, model) in models.iter() {
        if let Some(expand_info) = &model.expand_info {
            for child_id in expand_info.children.iter() {
                if let Some(reference_count) = reference_counts.get_mut(child_id) {
                    *reference_count += 1;
                } else {
                    eprintln!(
                        "错误：在更新reference count时在reference_counts中未找到子节点{}",
                        child_id
                    );
                }
            }
        }
    }
    for (id, reference_count) in reference_counts.iter() {
        println!("更新节点{}的reference count为{}", id, reference_count);
        models.get_mut(id).unwrap().ref_count = *reference_count;
    }
}

fn replace_node_and_update_children(
    id: u64,
    new_id: Option<u64>,
    models: &mut BTreeMap<u64, Model>,
) -> Result<HashSet<u64>, String> {
    println!("删除节点：{}", id);
    models
        .remove(&id)
        .ok_or(format!("未找到要删除的模型：{}", id))?;
    let mut ids_to_update = HashSet::new();
    for (_, model) in models.iter_mut() {
        if let Some(expand_info) = model.expand_info.as_mut() {
            if expand_info.children.contains(&id) {
                if let Some(new_id) = new_id {
                    expand_info.children.iter_mut().for_each(|child_id| {
                        if *child_id == id {
                            *child_id = new_id;
                        }
                    });
                } else {
                    expand_info.children.retain(|child_id| *child_id != id);
                }
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
    let mut state = state.write().unwrap();
    println!(
        "Rust: request_rename called with id: {}, new_name: {}",
        id, new_name
    );
    let models = &mut state
        .curr_tree_model
        .as_mut()
        .ok_or("模型未加载".to_string())?
        .models;
    let new_name_owner = models.iter().find(|(_, model)| model.name == new_name);
    let new_name_owner_id = new_name_owner.map(|(id, _)| *id);
    let model = models.get_mut(&id).ok_or(format!("未找到模型{}", id))?;
    if new_name == "" {
        Err("重命名失败：新名称为空".to_string())?;
    }
    if model.name == new_name {
        Err("重命名失败：新名称与旧名称相同".to_string())?;
    }
    if let Some(new_name_owner_id) = new_name_owner_id {
        if model.expand_info.is_some() {
            Err("重命名失败：新名称已存在".to_string())?;
        }
        let mut ids_to_update =
            replace_node_and_update_children(id, Some(new_name_owner_id), models)?;
        ids_to_update.insert(new_name_owner_id);
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

fn request_delete_helper(
    id: u64,
    parent: Option<u64>,
    state: tauri::State<RwLock<TauriState>>,
) -> Result<DeleteResponse, String> {
    println!(
        "Rust: request_delete called with id: {}, parent: {:?}",
        id, parent
    );
    if parent.is_none() {
        assert_eq!(id, 0);
        Err("根节点不可删除".to_string())?;
    }
    let mut state = state.write().unwrap();
    let models = &mut state
        .curr_tree_model
        .as_mut()
        .ok_or("模型未加载".to_string())?
        .models;

    // if the reference count is 1, we can delete the model, unless it is root
    let model = models.get_mut(&id).ok_or(format!("未找到模型{}", id))?;
    if model.ref_count == 1 && id != 0 {
        let ids_to_update = replace_node_and_update_children(id, None, models)?;
        Ok(DeleteResponse {
            id_to_remove: Some(id),
            ids_to_update: ids_to_update.into_iter().collect(),
        })
    } else {
        let parent_model = models
            .get_mut(&parent.unwrap())
            .ok_or(format!("未找到模型{}", parent.unwrap()))?;
        let parent_children = &mut parent_model
            .expand_info
            .as_mut()
            .ok_or("父节点无子节点".to_string())?
            .children;
        parent_children.remove(
            parent_children
                .iter()
                .position(|x| *x == id)
                .ok_or("未找到要删除的模型")?,
        );
        update_reference_count(models);
        Ok(DeleteResponse {
            id_to_remove: None,
            ids_to_update: vec![parent.unwrap(), id],
        })
    }
}

#[tauri::command]
pub fn request_delete(
    id: u64,
    parent: Option<u64>,
    state: tauri::State<RwLock<TauriState>>,
) -> MyResult<DeleteResponse, String> {
    let result = request_delete_helper(id, parent, state);
    match result {
        Ok(id) => MyResult::Ok(id),
        Err(e) => MyResult::Err(e),
    }
}

fn request_add_helper(id: u64, state: tauri::State<RwLock<TauriState>>) -> Result<u64, String> {
    let mut state = state.write().unwrap();
    let tree_model = state
        .curr_tree_model
        .as_mut()
        .ok_or("添加错误：模型未加载".to_string())?;
    let model = tree_model
        .models
        .get_mut(&id)
        .ok_or(format!("添加错误：未找到模型{}", id))?;
    let expand_info = model
        .expand_info
        .as_mut()
        .ok_or("添加失败：模型无子节点".to_string())?;
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
pub fn request_add(id: u64, state: tauri::State<RwLock<TauriState>>) -> MyResult<u64, String> {
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
    app: AppHandle,
) -> Result<u64, String> {
    println!("Rust: request_can_expand_toggling called with id: {}", id);
    let mut state = state.write().unwrap();
    let tree_model = state
        .curr_tree_model
        .as_mut()
        .ok_or("模型未加载".to_string())?;
    let model = tree_model
        .models
        .get_mut(&id)
        .ok_or(format!("未找到模型{}", id))?;
    if let Some(expand_info) = &model.expand_info {
        // ask user if confirm the operation
        if !expand_info.children.is_empty() {
            let answer = app
                .dialog()
                .message("当前节点下仍有子节点，是否清空所有子节点？")
                .title("清空子节点确认")
                .buttons(MessageDialogButtons::OkCancelCustom(
                    "确认".to_string(),
                    "取消".to_string(),
                ))
                .blocking_show();
            if answer {
                model.expand_info = None;
            } else {
                Err("已取消清空子节点".to_string())?;
            }
        } else {
            model.expand_info = None;
        }
    } else {
        model.expand_info = Some(ExpandInfo {
            algorithm: Algorithm::None,
            children: vec![],
        });
    }
    Ok(id)
}

#[tauri::command]
pub fn request_can_expand_toggling(
    id: u64,
    state: tauri::State<RwLock<TauriState>>,
    app: AppHandle,
) -> MyResult<u64, String> {
    let result = request_can_expand_toggling_helper(id, state, app);
    match result {
        Ok(id) => MyResult::Ok(id),
        Err(e) => MyResult::Err(e),
    }
}

fn request_save_helper(state: tauri::State<RwLock<TauriState>>) -> Result<(), String> {
    println!("Rust: request_save called");
    let state = state.read().unwrap();
    let tree_model = state
        .curr_tree_model
        .as_ref()
        .ok_or("保存错误：模型未加载".to_string())?;
    let file_path = state
        .curr_file_path
        .as_ref()
        .ok_or("保存错误：文件路径未加载".to_string())?;
    // let result = crate::saver::save_models(file_path, tree_model)?;
    let root_name = tree_model.models.get(&0).unwrap().name.clone();
    let mut file_models = HashMap::<u64, FileModel>::new();
    let mut met = HashSet::<u64>::new();
    let mut queue = VecDeque::<u64>::new();
    queue.push_back(0);
    let mut counter = 0;
    while let Some(id) = queue.pop_front() {
        counter += 1;
        if counter > 10000 {
            Err("在保存时遇到错误：循环次数过多".to_string())?;
        }
        if met.contains(&id) {
            continue;
        }
        met.insert(id);
        if file_models.get(&id).is_some() {
            Err(format!("在保存时遇到错误：重复的模型{}", id))?;
        }
        let model = tree_model
            .models
            .get(&id)
            .ok_or(format!("在保存时遇到错误：未找到模型{}", id))?;
        if model.expand_info.is_none() {
            // 不保存没有子节点的节点
            continue;
        }
        let ExpandInfo {
            children,
            algorithm,
        } = model.expand_info.clone().unwrap();
        for child in children.iter() {
            queue.push_back(*child);
        }
        let children_names = children
            .iter()
            .map(|child_id| {
                let model = tree_model.models.get(&child_id).ok_or(format!(
                    "保存时遇到错误：取children_names时遇到未知模型：{}",
                    child_id
                ))?;
                Ok(model.name.clone())
            })
            .collect::<Result<Vec<String>, String>>()?;
        let file_model = FileModel {
            name: model.name.clone(),
            algorithm: algorithm.to_string(),
            children: children_names,
        };
        file_models.insert(id, file_model);
        queue.extend(children);
    }
    let file_models = file_models
        .into_iter()
        .map(|(_id, file_model)| file_model)
        .collect::<Vec<_>>();
    let file_tree_model = FileTreeModel {
        root_name,
        data: file_models,
    };
    save_models(file_path, file_tree_model)?;
    Ok(())
}

#[tauri::command]
pub fn request_save(state: tauri::State<RwLock<TauriState>>) -> MyResult<(), String> {
    let result = request_save_helper(state);
    match result {
        Ok(_) => MyResult::Ok(()),
        Err(e) => MyResult::Err(e),
    }
}

fn request_calculate_helper(
    app: AppHandle,
    state: tauri::State<RwLock<TauriState>>,
) -> Result<(), String> {
    println!("Rust: request_calculate called");
    let file_path = app.dialog().file().blocking_pick_file();
    let file_path = file_path
        .map(|path| match path {
            FilePath::Path(pathbuf) => pathbuf.to_string_lossy().to_string(),
            FilePath::Url(url) => url.to_string(),
        })
        .ok_or("未选择文件".to_string())?;
    let file_data = load_data(&file_path)?;
    let mut state = state.write().unwrap();
    let tree_model = state
        .curr_tree_model
        .as_ref()
        .ok_or("模型未加载".to_string())?;
    let models = &tree_model.models;
    let name_to_id = models
        .iter()
        .filter(|(_name, model)| model.expand_info.is_none())
        .map(|(id, model)| (model.name.clone(), *id))
        .collect::<HashMap<String, u64>>();
    let required_names = name_to_id
        .iter()
        .map(|(name, _id)| name.clone())
        .collect::<HashSet<String>>();
    for required_name in required_names.iter() {
        if !file_data.contains_key(required_name) {
            Err(format!("计算失败：文件中缺少数据{}", required_name))?;
        }
    }
    // create a dynamic programming mem for calculation
    let mut mem = HashMap::<u64, f64>::new();
    for (name, id) in name_to_id.iter() {
        mem.insert(*id, file_data.get(name).unwrap().clone());
    }
    // create a helper function for calculating a node's value based on its childrens'
    // this function will be called recursively
    // do not modify the model's value directly, instead, store it in mem
    fn calculate(
        models: &BTreeMap<u64, Model>,
        mem: &mut HashMap<u64, f64>,
        id: u64,
    ) -> Result<f64, String> {
        if let Some(value) = mem.get(&id) {
            return Ok(*value);
        }
        let model = models
            .get(&id)
            .ok_or(format!("计算失败：未找到模型{}", id))?;
        let expand_info = model
            .expand_info
            .as_ref()
            .ok_or("计算失败：模型既无子节点也没有现成的值".to_string())?;
        let mut children_values = vec![];
        for child_id in expand_info.children.iter() {
            children_values.push(calculate(models, mem, *child_id)?);
        }
        let algorithm = expand_info.algorithm.clone();
        if let Algorithm::None = algorithm {
            println!("计算失败：模型{}的算法为None", id);
            Err(format!("计算失败：模型{}的算法为None", id))?;
        }
        let value = algorithm.calculate(&children_values);
        println!("计算了{}的值：{}", id, value);
        mem.insert(id, value);
        Ok(value)
    }
    let _ = calculate(models, &mut mem, 0)?;
    // update the model's value
    let models = &mut state.curr_tree_model.as_mut().unwrap().models;
    for (id, value) in mem.iter() {
        let model = models
            .get_mut(id)
            .ok_or(format!("计算失败：未找到模型{}", id))?;
        model.value = Some(*value);
    }
    Ok(())
}

#[tauri::command]
pub fn request_calculate(
    app: AppHandle,
    state: tauri::State<RwLock<TauriState>>,
) -> MyResult<(), String> {
    let result = request_calculate_helper(app, state);
    println!("request_calculate result: {:?}", result);
    match result {
        Ok(_) => MyResult::Ok(()),
        Err(e) => MyResult::Err(e),
    }
}

fn query_values_helper(
    ids: Vec<u64>,
    state: tauri::State<RwLock<TauriState>>,
) -> Result<QueryValuesResponse, String> {
    println!("Rust: query_values called");
    let state = state.read().unwrap();
    let models = &state
        .curr_tree_model
        .as_ref()
        .ok_or("模型未加载".to_string())?
        .models;
    let mut values = HashMap::new();
    for id in ids.iter() {
        let model = models.get(id).ok_or(format!("未找到模型{}", id))?;
        let value = model.value.ok_or(format!("模型{}的值未计算", id))?;
        values.insert(id.to_string(), value);
    }
    Ok(QueryValuesResponse { values })
}

#[tauri::command]
pub fn query_values(
    ids: Vec<u64>,
    state: tauri::State<RwLock<TauriState>>,
) -> MyResult<QueryValuesResponse, String> {
    let result = query_values_helper(ids, state);
    match result {
        Ok(values) => MyResult::Ok(values),
        Err(e) => MyResult::Err(e),
    }
}

fn request_template_generation_helper(app: AppHandle, state: tauri::State<RwLock<TauriState>>) -> Result<(), String> {
    println!("Rust: request_template_generation called");
    let state = state.read().unwrap();
    let tree_model = state
        .curr_tree_model
        .as_ref()
        .ok_or("模型未加载".to_string())?;
    let models = &tree_model.models;
    let mut file_data = FileData::new();
    for (_id, model) in models.iter() {
        if model.expand_info.is_none() {
            file_data.insert(model.name.clone(), 0.0);
        }
    }
    let file_data_str = serde_json::to_string(&file_data).unwrap();
    let file_path = app
    .dialog()
    .file()
    .set_file_name("template.json")
    // .add_filter("My Filter", &["png", "jpeg"])
    .blocking_save_file();
    // write str to file path
    let file_path = file_path
        .map(|path| match path {
            FilePath::Path(pathbuf) => pathbuf.to_string_lossy().to_string(),
            FilePath::Url(url) => url.to_string(),
        })
        .ok_or("未选择文件".to_string())?;
    std::fs::write(file_path, file_data_str).unwrap();
    Ok(())
}

#[tauri::command]
pub fn request_template_generation(app: AppHandle, state: tauri::State<RwLock<TauriState>>) -> MyResult<(), String> {
    let result = request_template_generation_helper(app, state);
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
