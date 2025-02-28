use dirs::home_dir;
// use k8s_openapi::api::core::v1::Secret;
use oci_sdk::{config::AuthConfig, identity::Identity, vault_secret::Vault};
// use serde::Deserialize;
// use std::collections::HashMap;

// use crate::secrets::secret;

// use crate::secrets::secret;
// use reqwest::Response;

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
        // secret: Secret,
        oracle: &crate::secrets::cluster_secret_store::OracleProvider,
        external_secret: crate::secrets::external_secret::ExternalSecret,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let secret_name = external_secret.spec.data_from[0].extract.key.as_str();
        // println!("{:?}", secret);

        let vault_secret_provider = Vault::new(self.get_identity());

        self.iterate_over_secrets_paths(secret_name, &oracle.vault, vault_secret_provider)
            .await?;
        Ok(())
    }

    async fn iterate_over_secrets_paths(
        &self,
        secret_name: &str,
        vault_ocid: &str,
        vault_secret_provider: Vault,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let secret_content = vault_secret_provider
            .get_secret(secret_name, vault_ocid)
            .await?;

        println!("{:?}", secret_content.get_json().await?);
        Ok(())
    }
}
