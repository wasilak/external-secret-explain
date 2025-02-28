use aws_config::{self, Region};
use aws_sdk_secretsmanager;
use k8s_openapi::api::core::v1::Secret;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AWSProvider {}

impl AWSProvider {
    pub fn new() -> Self {
        AWSProvider {}
    }

    pub async fn handle(
        &self,
        secret: Secret,
        external_secret: crate::secrets::external_secret::ExternalSecret,
        region: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match secret.data {
            Some(data) => {
                for (key, value) in data {
                    let decoded_value = String::from_utf8(value.0).unwrap();
                    println!("{}: {}", key, decoded_value);
                }
            }
            None => println!("No data found in secret"),
        }

        let secrets_names: Vec<String> = external_secret
            .spec
            .data_from
            .iter()
            .map(|d| d.extract.key.clone())
            .collect();

        println!("Secrets names: {:?}", secrets_names);
        let secrets_paths_with_keys = self
            .iterate_over_secrets_paths(&secrets_names, region)
            .await?;

        println!("{:?}", secrets_paths_with_keys);

        Ok(())
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
                    println!("No secrets found. Are you sure you are using proper AWS auth?");
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
            Err(e) => println!("Error getting secrets. Please verify AWS auth: {}", e),
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

        let response = client
            .batch_get_secret_value()
            .set_secret_id_list(Some(secret_names.clone()))
            .send()
            .await?;

        let secrets = response
            .secret_values()
            .iter()
            .filter_map(|secret| {
                println!("{:?}", secret);
                Some((
                    secret.name().unwrap_or_default().to_string(),
                    secret.secret_string().unwrap_or_default().to_string(),
                ))
            })
            .collect();

        Ok(secrets)
    }
}
