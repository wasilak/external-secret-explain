mod k8s;
mod providers;
mod secrets;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of Kubernetes secret
    #[arg()]
    secret_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct FinalResult {
    name: String,
    data_from: Vec<String>,
    provider: secrets::cluster_secret_store::ClusterSecretStoreSpec,
    secrets: Option<Vec<providers::common::MatchedKey>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions::default()).await?;

    let k8s_secret = k8s::Wrapper::get_secret(&config, &args.secret_name).await?;
    let k8s_secret_data = k8s::Wrapper::get_secret_data(&k8s_secret).await?;
    let external_secret =
        secrets::external_secret::get(&config, &secrets::secret::get_owner(&k8s_secret)).await?;

    let data_from = k8s::Wrapper::get_external_secret_data_from(external_secret.clone());

    let cluster_secret_store = secrets::cluster_secret_store::get(
        config,
        &external_secret.clone().spec.secret_store_ref.unwrap().name,
    )
    .await?;

    let mut final_result = FinalResult {
        name: args.secret_name,
        data_from: data_from.clone(),
        provider: cluster_secret_store.spec.clone(),
        secrets: None,
    };

    match &cluster_secret_store.spec.provider.kind {
        secrets::cluster_secret_store::ProviderType::Aws(aws_ref) => {
            let provider = providers::aws::AWSProvider::new();
            let matched_keys = provider
                .handle(&k8s_secret_data, &data_from, &aws_ref.region)
                .await;

            final_result.secrets = Some(matched_keys.unwrap());
        }
        secrets::cluster_secret_store::ProviderType::Gcp(_) => (),
        secrets::cluster_secret_store::ProviderType::Oracle(oracle) => {
            let provider = providers::oracle::OracleProvider::new();
            let matched_keys = provider
                .handle(&k8s_secret_data, &data_from, &oracle)
                // .handle(secret, &oracle, external_secret.clone())
                .await;

            final_result.secrets = Some(matched_keys.unwrap());
        }
    };

    println!(
        "{}",
        match serde_yaml::to_string(&final_result) {
            Ok(s) => s,
            Err(e) => e.to_string(),
        }
    );

    Ok(())
}
