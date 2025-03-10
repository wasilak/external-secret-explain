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
    Oracle(OracleProvider),
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

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OracleProvider {
    pub auth: OracleAuth,
    pub principal_type: String,
    pub region: String,
    pub vault: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OracleAuth {
    pub secret_ref: OracleSecretRef,
    pub tenancy: String,
    pub user: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OracleSecretRef {
    pub fingerprint: OracleSecretKeyRef,
    pub privatekey: OracleSecretKeyRef,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OracleSecretKeyRef {
    pub key: String,
    pub name: String,
    pub namespace: String,
}

pub async fn get(
    config: Config,
    name: &str,
) -> Result<ClusterSecretStore, Box<dyn std::error::Error>> {
    let client = match kube::Client::try_from(config) {
        Ok(client) => client,
        Err(e) => return Err(Box::new(e)),
    };
    let api: Api<ClusterSecretStore> = Api::all(client);

    match api.get(name).await {
        Ok(external_secret) => return Ok(external_secret),
        Err(e) => return Err(Box::new(e)),
    };
}
