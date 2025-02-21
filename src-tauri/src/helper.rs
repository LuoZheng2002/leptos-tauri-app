use shared::Model;
use std::collections::HashMap;

pub fn suggest_new_name_dupe(new_name: &str, models: &HashMap<String, Model>) -> String {
    let mut new_name = new_name.to_string();
    while models.get(&new_name).is_some() {
        new_name = format!("{}（错误：重名）", new_name);
    }
    new_name
}
pub fn suggest_new_name_add(models: &HashMap<String, Model>) -> String {
    let mut new_name = "新节点".to_string();
    let mut i = 0;
    while models.get(&new_name).is_some() {
        i += 1;
        new_name = format!("新节点{}", i);
    }
    new_name
}
