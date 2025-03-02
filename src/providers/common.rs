use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MatchedKey {
    key: String,
    value: String,
    source: String,
}

pub fn match_secret_keys(
    k8s_secret: &HashMap<String, String>,
    external_secrets_paths_with_keys: HashMap<String, Vec<String>>,
    data_from: &Vec<String>,
) -> Vec<MatchedKey> {
    let mut matched_keys: HashMap<String, MatchedKey> = HashMap::new();

    for (k, v) in k8s_secret.iter() {
        for path in data_from {
            let keys = match external_secrets_paths_with_keys.get(path) {
                Some(keys) => keys,
                None => {
                    continue;
                }
            };

            if keys.contains(k) && matched_keys.get(k).is_none() {
                let matched_key = MatchedKey {
                    key: k.clone(),
                    value: v.clone(),
                    source: path.clone(),
                };
                matched_keys.insert(k.clone(), matched_key);
            }
        }
    }

    // HashMap Was used to deduplicate the keys found while traversing merge order, but ultimately we need to return a Vec
    matched_keys.into_iter().map(|(_, v)| v).collect()
}
