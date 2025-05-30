mod k8s;
mod providers;
mod secrets;
mod utils;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of Kubernetes secret
    #[arg()]
    secret_name: String,

    /// Output format
    #[arg(short, long, default_value = "yaml")]
    output: OutputFormat,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Yaml,
    Json,
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

    let _ = rustls::crypto::ring::default_provider().install_default();

    utils::setup_logging();

    let config = kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions::default())
        .await
        .unwrap_or_else(|e| {
            error!("Error getting kubeconfig: {}", e.to_string());
            std::process::exit(1);
        });

    let k8s_secret = k8s::Wrapper::get_secret(&config, &args.secret_name)
        .await
        .unwrap_or_else(|e| {
            error!("Error getting secret: {}", e.to_string());
            std::process::exit(1);
        });

    let k8s_secret_data = k8s::Wrapper::get_secret_data(&k8s_secret)
        .await
        .unwrap_or_else(|e| {
            error!("Error getting secret data: {}", e.to_string());
            std::process::exit(1);
        });

    let external_secret =
        secrets::external_secret::get(&config, &secrets::secret::get_owner(&k8s_secret))
            .await
            .unwrap_or_else(|e| {
                error!("Error getting external secret: {}", e.to_string());
                std::process::exit(1);
            });

    let data_from = k8s::Wrapper::get_external_secret_data_from(external_secret.clone());

    let cluster_secret_store = secrets::cluster_secret_store::get(
        config,
        &external_secret.clone().spec.secret_store_ref.unwrap().name,
    )
    .await
    .unwrap_or_else(|e| {
        error!(
            "Error getting cluster secret store
        : {}",
            e.to_string()
        );
        std::process::exit(1);
    });

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

    let output = match args.output {
        OutputFormat::Yaml => match serde_yaml::to_string(&final_result) {
            Ok(s) => s,
            Err(e) => e.to_string(),
        },
        OutputFormat::Json => match serde_json::to_string(&final_result) {
            Ok(s) => s,
            Err(e) => e.to_string(),
        },
    };

    println!("{}", output);

    Ok(())
}
