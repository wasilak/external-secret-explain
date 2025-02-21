use k8s_openapi::api::core::v1::Secret;
use kube::{Api, Client, Config};

pub async fn get(config: Config, secret_name: &str) -> Result<Secret, Box<dyn std::error::Error>> {
    let client: Client = kube::Client::try_from(config)?;
    let api: Api<Secret> = Api::default_namespaced(client);

    let secret = api.get(secret_name).await?;
    Ok(secret)
}

pub fn get_owner(secret: &Secret) -> String {
    if let Some(owner_references) = &secret.metadata.owner_references {
        if let Some(owner_reference) = owner_references.get(0) {
            if owner_reference.kind == "ExternalSecret" {
                return owner_reference.name.to_string();
            }
        }
    }
    "".to_string()
}
