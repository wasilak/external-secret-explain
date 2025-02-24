// use kube::api::ListParams;
// use kube::{Client};
use kube::{Api, Config};
use kube_derive::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    group = "external-secrets.io",
    version = "v1beta1",
    kind = "ExternalSecret",
    namespaced
)]
pub struct Spec {
    pub secret_store_ref: Option<SecretStoreRef>,
    pub data_from: Vec<DataFrom>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct SecretStoreRef {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct DataFrom {
    pub extract: ExtractRef,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtractRef {
    pub conversion_strategy: String,
    pub decoding_strategy: String,
    pub key: String,
}

pub async fn get(config: Config, name: &str) -> Result<ExternalSecret, Box<dyn std::error::Error>> {
    let client = kube::Client::try_from(config)?;
    let api: Api<ExternalSecret> = Api::default_namespaced(client);

    let external_secret = api.get(name).await?;
    Ok(external_secret)
}

// pub async fn list(config: Config) -> Result<(), Box<dyn std::error::Error>> {
//     let client: Client = kube::Client::try_from(config)?;
//     let secrets: Api<ExternalSecret> = Api::default_namespaced(client);

//     let list_params = ListParams::default();

//     for secret in secrets.list(&list_params).await? {
//         println!("{:?}", secret.metadata.name.unwrap_or_default());
//         println!(
//             "secret_store_ref: {:?}",
//             secret
//                 .spec
//                 .secret_store_ref
//                 .as_ref()
//                 .map(|ref_name| ref_name.name.clone())
//                 .unwrap_or_else(|| "no-ref".to_string())
//         );
//     }

//     Ok(())
// }
