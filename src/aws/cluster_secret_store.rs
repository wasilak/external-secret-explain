use kube::{Api, Config};
use kube_derive::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    group = "external-secrets.io",
    version = "v1beta1",
    kind = "ClusterSecretStore",
    namespaced
)]
pub struct ClusterSecretStoreSpec {
    pub provider: Provider,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    #[serde(flatten)]
    pub kind: ProviderType,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ProviderType {
    Aws(AWSProvider),
    Gcp(GCPProvider),
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AWSProvider {
    pub region: String,
    pub role: String,
    pub service: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GCPProvider {
    pub project_id: String,
    pub auth_secret_ref: String,
}

pub async fn get(
    config: Config,
    name: &str,
) -> Result<ClusterSecretStore, Box<dyn std::error::Error>> {
    let client = kube::Client::try_from(config)?;
    let api: Api<ClusterSecretStore> = Api::all(client);

    let external_secret = api.get(name).await?;
    Ok(external_secret)
}
