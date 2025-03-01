use std::fs;

use crate::models::FileTreeModel;

pub fn save_models(file_path: &str, file_tree_model: FileTreeModel) -> Result<(), String> {
    let content = serde_json::to_string(&file_tree_model)
        .map_err(|e| format!("序列化模型文件错误\n{}", e))?;
    fs::write(file_path, content).map_err(|e| format!("写入模型文件错误\n{}", e))?;
    Ok(())
}
