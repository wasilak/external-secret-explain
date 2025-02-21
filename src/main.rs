mod aws;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let secret_name = "external-secret-observability";

    let args: Vec<String> = env::args().collect();
    let secret_name = args.get(1).expect("Usage: program <secret_name>");

    println!("Fetching details for secret: {}", secret_name);

    let config = kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions::default()).await?;

    // aws::external_secret::list(config).await?;

    let secret = aws::secret::get(config.clone(), secret_name).await?;

    let external_secret =
        aws::external_secret::get(config.clone(), &aws::secret::get_owner(&secret)).await?;

    let cluster_secret_store = aws::cluster_secret_store::get(
        config,
        &external_secret.spec.secret_store_ref.unwrap().name,
    )
    .await?;

    println!(
        "{}",
        serde_yaml::to_string(&cluster_secret_store.spec).unwrap()
    );

    let provider_name = match &cluster_secret_store.spec.provider.kind {
        aws::cluster_secret_store::ProviderType::Aws(_) => "aws",
        aws::cluster_secret_store::ProviderType::Gcp(_) => "gcp",
    };
    println!("🔍 Secret Store Provider: {}", provider_name);

    // 1. get provider name from cluster_secret_store
    // 2. access provder and get secrets according to external_secret.spec.data_from
    // 3. get secret keys from secret.data
    // 4. figure out from which secret in merge list comes each key value
    // 5. output secret.data as YAML annotated with secret store path from which it comes

    Ok(())
}
