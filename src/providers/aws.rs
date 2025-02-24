use aws_config;
use aws_sdk_secretsmanager;
use k8s_openapi::api::core::v1::Secret;
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        match secret.data {
            Some(data) => {
                for (key, value) in data {
                    let decoded_value = String::from_utf8(value.0).unwrap();
                    println!("{}: {}", key, decoded_value);
                }
                println!("Found secret data");
            }
            None => println!("No data found in secret"),
        }

        let secrets_names: Vec<String> = external_secret
            .spec
            .data_from
            .iter()
            .map(|d| d.extract.key.clone())
            .collect();

        self.iterate_over_secrets_paths(&secrets_names).await?;

        Ok(())
    }

    pub async fn get_secrets(
        &self,
        secret_names: &Vec<String>,
    ) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
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
                Some((
                    secret.name().unwrap_or_default().to_string(),
                    secret.secret_string().unwrap_or_default().to_string(),
                ))
            })
            .collect();

        Ok(secrets)
    }

    async fn iterate_over_secrets_paths(
        &self,
        secrets: &Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.get_secrets(secrets).await {
            Ok(secrets) => {
                // println!("{:?}", &secrets);

                for secret in secrets {
                    let parsed_secret: serde_json::Value = serde_json::from_str(&secret.1).unwrap();
                    println!("\n{}:", secret.0);

                    for (key, value) in parsed_secret.as_object().unwrap() {
                        println!("{}{}", key, value);
                    }
                }
            }
            Err(e) => println!("Error getting secrets: {}", e),
        }
        Ok(())
    }
}
