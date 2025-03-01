use crate::models::{FileData, FileModel, FileTreeModel, TreeModel};
use rand::Rng;
use shared::{Algorithm, ExpandInfo, Model};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

pub fn load_models(file_path: &str, randomize_algorithm: bool) -> Result<TreeModel, String> {
    let counter = AtomicU64::new(0);
    // Reset counter
    counter.store(0, Ordering::Relaxed);
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("读取模型文件{:?}错误\n{}", file_path, e))?;
    let file_tree_model = serde_json::from_str::<FileTreeModel>(&content)
        .map_err(|e| format!("解析模型文件{:?}错误\n{}", file_path, e))?;
    let models = file_tree_model.data;
    let root_name = file_tree_model.root_name;
    let models: BTreeMap<String, FileModel> = models
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
    let mut rng = rand::rng();
    // 转换模型，加入id
    let mut models: BTreeMap<u64, Model> = name_to_id
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
                let algorithm_str = model.algorithm.clone();
                let mut algorithm_enum = algorithm_str.parse().unwrap_or(Algorithm::None);
                if matches!(algorithm_enum, Algorithm::None) && randomize_algorithm {
                    algorithm_enum = Algorithm::random(rng.random());
                }
                println!(
                    "字符串算法：{}, 枚举算法：{:?}",
                    algorithm_str, algorithm_enum
                );
                let model = Model {
                    id: *id,
                    name: model.name.clone(),
                    ref_count: 0,
                    expand_info: Some(ExpandInfo {
                        algorithm: algorithm_enum,
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

pub fn load_data(file_path: &str) -> Result<FileData, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("读取数据文件{:?}错误\n{}", file_path, e))?;
    let file_data = serde_json::from_str::<FileData>(&content)
        .map_err(|e| format!("解析数据文件{:?}错误\n{}", file_path, e))?;
    Ok(file_data)
}
