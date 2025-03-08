use super::common::{match_secret_keys, MatchedKey};
use dirs::home_dir;
use oci_sdk::{config::AuthConfig, identity::Identity, vault_secret::Vault};
use std::collections::HashMap;
use tracing::error;

#[derive(Clone)]
pub struct OracleProvider {
    identity: Identity,
}

impl OracleProvider {
    pub fn new() -> Self {
        let config_path = home_dir()
            .map(|p| p.join(".oci/config"))
            .expect("Could not determine home directory");

        // Set up authentication configuration
        let auth_config = AuthConfig::from_file(
            Some(config_path.to_str().unwrap().to_string()), // Path to your OCI config file
            Some("DEFAULT".to_string()),                     // Profile name
        );

        // Create an Identity service client
        let identity = Identity::new(auth_config, None);
        OracleProvider { identity }
    }

    pub fn get_identity(&self) -> Identity {
        self.identity.clone()
    }

    pub async fn handle(
        &self,
        k8s_secret_data: &HashMap<String, String>,
        data_from: &Vec<String>,
        oracle: &crate::secrets::cluster_secret_store::OracleProvider,
    ) -> Result<Vec<MatchedKey>, Box<dyn std::error::Error>> {
        let vault_secret_provider = Vault::new(self.get_identity());
        let external_secrets_paths_with_keys = match self
            .iterate_over_secrets_paths(&data_from, &oracle.vault, vault_secret_provider)
            .await
        {
            Ok(secrets) => secrets,
            Err(e) => return Err(e),
        };

        let matched_keys = match_secret_keys(
            &k8s_secret_data,
            external_secrets_paths_with_keys,
            &data_from,
        );

        Ok(matched_keys)
    }

    async fn iterate_over_secrets_paths(
        &self,
        secrets: &Vec<String>,
        vault_ocid: &str,
        vault_secret_provider: Vault,
    ) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
        let mut secrets_paths_with_keys: HashMap<String, Vec<String>> = HashMap::new();

        for secret_name in secrets {
            match vault_secret_provider
                .get_secret(secret_name, vault_ocid)
                .await
            {
                Ok(secret) => {
                    let secret_content = match secret.get_json().await {
                        Ok(content) => content,
                        Err(e) => {
                            error!("Error getting secret content: {}", e);
                            continue;
                        }
                    };

                    let secret_keys = secret_content.keys().cloned().collect();
                    secrets_paths_with_keys.insert(secret_name.to_string(), secret_keys);
                }
                Err(e) => {
                    error!("Error getting secret: {}", e);
                    continue;
                }
            };
        }
        Ok(secrets_paths_with_keys)
    }
}
