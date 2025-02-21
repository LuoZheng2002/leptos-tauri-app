use crate::models::{FileModel, FileTreeModel, TreeModel};
use shared::{Algorithm, ExpandInfo, Model};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

fn parse_algorithm(algo: &str) -> Algorithm {
    let algorithm = match algo {
        "求和" => Algorithm::Sum,
        "求乘积" => Algorithm::Product,
        "求平均" => Algorithm::Average,
        "求最大" => Algorithm::Max,
        "求最小" => Algorithm::Min,
        _ => Algorithm::None,
    };
    algorithm
}

pub fn load_models(file_path: &str) -> Result<TreeModel, String> {
    let counter = AtomicU64::new(0);
    // Reset counter
    counter.store(0, Ordering::Relaxed);
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("读取模型文件{:?}错误\n{}", file_path, e))?;
    let file_tree_model = serde_json::from_str::<FileTreeModel>(&content)
        .map_err(|e| format!("解析模型文件{:?}错误\n{}", file_path, e))?;
    let models = file_tree_model.data;
    let root_name = file_tree_model.root_name;
    let models: HashMap<String, FileModel> = models
        .into_iter()
        .map(|model| (model.name.clone(), model))
        .collect();
    // 遍历所有模型及其children，将所有名字给一个独一无二的编号
    let mut name_to_id = HashMap::<String, u64>::new();
    // 确保root_name在模型中
    models
        .get(&root_name)
        .ok_or(format!("模型文件中没有根节点\"{}\"", root_name))?;
    // 将0赋值给root节点
    let root_id = counter.fetch_add(1, Ordering::Relaxed);
    assert!(root_id == 0);
    name_to_id.insert(root_name.to_string(), root_id);
    models.iter().for_each(|(_name, model)| {
        name_to_id
            .entry(model.name.clone())
            .or_insert(counter.fetch_add(1, Ordering::Relaxed));
        model.children.iter().for_each(|child| {
            name_to_id
                .entry(child.clone())
                .or_insert(counter.fetch_add(1, Ordering::Relaxed));
        });
    });
    // 转换模型，加入id
    let mut models: HashMap<u64, Model> = name_to_id
        .iter()
        .map(|(name, id)| match models.get(name) {
            Some(model) => {
                let children = model
                    .children
                    .iter()
                    .map(|child| {
                        name_to_id
                            .get(child)
                            .expect(
                                format!("child {} does not exist in name_to_id", child).as_str(),
                            )
                            .clone()
                    })
                    .collect();
                let model = Model {
                    id: *id,
                    name: model.name.clone(),
                    ref_count: 0,
                    expand_info: Some(ExpandInfo {
                        algorithm: parse_algorithm(&model.algorithm),
                        children,
                    }),
                    value: None,
                };
                (*id, model)
            }
            None => {
                let model = Model {
                    id: *id,
                    name: name.clone(),
                    ref_count: 0,
                    expand_info: None,
                    value: None,
                };
                (*id, model)
            }
        })
        .collect();

    // 记录所有模型的引用计数
    let mut ref_counts = models
        .iter()
        .map::<(u64, u64), _>(|(id, _model)| (*id, 0))
        .collect::<HashMap<u64, u64>>();
    models.iter().for_each(|(_name, model)| {
        if let Some(expand_info) = &model.expand_info {
            expand_info.children.iter().for_each(|child| {
                let count = ref_counts
                    .get(child)
                    .expect(format!("child {} does not exist in ref count", child).as_str());
                ref_counts.insert(child.clone(), count + 1);
            });
        }
    });
    // 加入ref_count
    models.iter_mut().for_each(|(_name, model)| {
        let count = ref_counts
            .get(&model.id)
            .expect(format!("model {} does not exist in ref count", model.name).as_str());
        model.ref_count = *count;
    });
    Ok(TreeModel {
        models,
        root_name,
        counter,
    })
}
