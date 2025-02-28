mod providers;
mod secrets;

use std::env;

mod k8s;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct FinalResult {
    name: String,
    data_from: Vec<String>,
    provider: secrets::cluster_secret_store::ClusterSecretStoreSpec,
    secrets: Option<Vec<providers::common::MatchedKey>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let secret_name = match args.get(1) {
        Some(secret_name) => secret_name,
        None => {
            eprintln!("Usage: {} <secret-name>", args[0]);
            std::process::exit(1);
        }
    };

    let config = kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions::default()).await?;

    let k8s_secret = k8s::Wrapper::get_secret(config.clone(), &secret_name).await?;
    let k8s_secret_data = k8s::Wrapper::get_secret_data(&k8s_secret).await?;
    let external_secret =
        secrets::external_secret::get(config.clone(), &secrets::secret::get_owner(&k8s_secret))
            .await?;

    let data_from = k8s::Wrapper::get_external_secret_data_from(external_secret.clone());

    let cluster_secret_store = secrets::cluster_secret_store::get(
        config,
        &external_secret.clone().spec.secret_store_ref.unwrap().name,
    )
    .await?;

    let mut final_result = FinalResult {
        name: secret_name.clone(),
        data_from: data_from.clone(),
        provider: cluster_secret_store.spec.clone(),
        secrets: None,
    };

    match &cluster_secret_store.spec.provider.kind {
        secrets::cluster_secret_store::ProviderType::Aws(aws_ref) => {
            let provider = providers::aws::AWSProvider::new();
            let matched_keys = provider
                .handle(k8s_secret_data, data_from, &aws_ref.region)
                .await;

            final_result.secrets = Some(matched_keys.unwrap());
        }
        secrets::cluster_secret_store::ProviderType::Gcp(_) => (),
        secrets::cluster_secret_store::ProviderType::Oracle(oracle) => {
            let provider = providers::oracle::OracleProvider::new();
            let _ = provider
                .handle(&oracle, external_secret.clone())
                // .handle(secret, &oracle, external_secret.clone())
                .await;
        }
    };

    println!("{}", serde_yaml::to_string(&final_result).unwrap());

    Ok(())
}
