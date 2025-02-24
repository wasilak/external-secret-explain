use dirs::home_dir;
// use kube::client::Body;
// use oci_sdk::base_client::{encode_body, oci_signer};
// use oci_sdk::base_client::oci_signer;
use oci_sdk::{config::AuthConfig, identity::Identity, vault_secret::VaultSecret};
// use reqwest::{Client, Method, RequestBuilder};
// use reqwest::Client;
use reqwest::Response;
// use chrono::{DateTime, Utc};

// use std::error::Error;
// use serde_json::json;
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

    // pub async fn get_secret(
    //     &self,
    //     secret_name: &str,
    // ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    //     let url = format!(
    //         "{}/20180608/secrets/{}/bundle",
    //         VAULT_BASE_URL.replace("{region}", OCI_REGION),
    //         secret_name
    //     );

    //     let client = Client::new();
    // }

    pub async fn get_secret(
        &self,
        secret_name: &str,
        vault_ocid: &str,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let vault_secret_provider = VaultSecret::new(self.get_identity());

        let response = vault_secret_provider
            .get_secret(secret_name, vault_ocid)
            .await?;
        return Ok(response);
    }
}

// 🔹 Function to Sign Requests Using OCI SDK
