use dirs::home_dir;
use k8s_openapi::api::core::v1::Secret;
use oci_sdk::{config::AuthConfig, identity::Identity, vault_secret::VaultSecret};
use reqwest::Response;

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

    pub async fn handle(
        &self,
        secret: Secret,
        oracle: &crate::secrets::cluster_secret_store::OracleProvider,
        external_secret: crate::secrets::external_secret::ExternalSecret,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // let vatul_ocid = "ocid1.vault.oc1.eu-frankfurt-1.entqnjjeaafoa.abtheljr7gcl5vu75z5kxvmm4nwbgr4wpgh5uvsgzlvhzkq4wabywkre446a";
        // let secret_name = String::from_str("loki").unwrap();
        let secret_name = external_secret.spec.data_from[0].extract.key.as_str();

        let result = self.get_secret(&secret_name, &oracle.vault).await?;
        let provider_secret = result.text().await?;
        println!("{:?}", provider_secret);

        println!("{:?}", secret);
        Ok(())
    }

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
