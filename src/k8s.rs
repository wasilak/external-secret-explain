use super::secrets;

use k8s_openapi::api::core::v1::Secret;
use kube_client::config::Config;
use std::collections::HashMap;

pub struct Wrapper {}

impl Wrapper {
    pub async fn get_secret(
        config: Config,
        secret_name: &str,
    ) -> Result<Secret, Box<dyn std::error::Error>> {
        let secret = secrets::secret::get(config, secret_name).await?;
        Ok(secret)
    }

    pub async fn get_secret_data(
        secret: &Secret,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut secret_data: HashMap<String, String> = HashMap::new();

        match &secret.data {
            Some(data) => {
                for (key, value) in data {
                    let decoded_value = String::from_utf8(value.0.clone()).unwrap();
                    secret_data.insert(key.clone(), decoded_value);
                }
            }
            None => println!("No data found in secret"),
        }

        Ok(secret_data)
    }

    pub fn get_external_secret_data_from(
        external_secret: crate::secrets::external_secret::ExternalSecret,
    ) -> Vec<String> {
        let mut data_from: Vec<String> = external_secret
            .spec
            .data_from
            .iter()
            .map(|d| d.extract.key.clone())
            .collect();

        // we will reverse the order of data_from to match the order of keys inheritence
        data_from.reverse();
        data_from
    }
}
