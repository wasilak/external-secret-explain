use super::common::{match_secret_keys, MatchedKey};
use aws_config::{self, Region};
use aws_sdk_secretsmanager;
use std::collections::HashMap;
use tracing::{error, warn};

#[derive(Clone)]
pub struct AWSProvider {}

impl AWSProvider {
    pub fn new() -> Self {
        AWSProvider {}
    }

    pub async fn handle(
        &self,
        k8s_secret_data: &HashMap<String, String>,
        data_from: &Vec<String>,
        region: &str,
    ) -> Result<Vec<MatchedKey>, Box<dyn std::error::Error>> {
        let external_secrets_paths_with_keys =
            match self.iterate_over_secrets_paths(&data_from, region).await {
                Ok(secrets) => secrets,
                Err(e) => return Err(e),
            };

        let matched_keys = match_secret_keys(
            &k8s_secret_data,
            external_secrets_paths_with_keys,
            data_from,
        );

        Ok(matched_keys)
    }

    async fn iterate_over_secrets_paths(
        &self,
        secrets: &Vec<String>,
        region: &str,
    ) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
        let mut secrets_paths_with_keys: HashMap<String, Vec<String>> = HashMap::new();
        match self.get_secrets(secrets, region).await {
            Ok(secrets) => {
                if secrets.is_empty() {
                    warn!("No secrets found. Are you sure you are using proper AWS auth?");
                    return Ok(secrets_paths_with_keys);
                }

                for secret in secrets {
                    let parsed_secret: serde_json::Value = serde_json::from_str(&secret.1).unwrap();

                    let mut secret_keys: Vec<String> = vec![];
                    for (key, _value) in parsed_secret.as_object().unwrap() {
                        secret_keys.push(key.to_string());
                    }

                    secrets_paths_with_keys.insert(secret.0, secret_keys);
                }
            }
            Err(e) => error!("Error getting secrets. Please verify AWS auth: {}", e),
        }
        Ok(secrets_paths_with_keys)
    }

    pub async fn get_secrets(
        &self,
        secret_names: &Vec<String>,
        region: &str,
    ) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(Region::new(region.to_string()))
            .load()
            .await;

        let client = aws_sdk_secretsmanager::Client::new(&config);

        let response = match client
            .batch_get_secret_value()
            .set_secret_id_list(Some(secret_names.clone()))
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => return Err(Box::new(e)),
        };

        let secrets = response
            .secret_values()
            .iter()
            .filter_map(|secret| {
                Some((
                    secret.name().unwrap_or_default().to_string(),
                    secret.secret_string().unwrap_or_default().to_string(),
                ))
            })
            .collect();

        Ok(secrets)
    }
}
