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

pub async fn get(
    config: &Config,
    name: &str,
) -> Result<ExternalSecret, Box<dyn std::error::Error>> {
    let client = match kube::Client::try_from(config.clone()) {
        Ok(client) => client,
        Err(e) => return Err(Box::new(e)),
    };
    let api: Api<ExternalSecret> = Api::default_namespaced(client);

    match api.get(name).await {
        Ok(external_secret) => return Ok(external_secret),
        Err(e) => return Err(Box::new(e)),
    };
}
