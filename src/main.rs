mod providers;
mod secrets;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let secret_name = "external-secret-observability";

    let args: Vec<String> = env::args().collect();
    let secret_name = match args.get(1) {
        Some(secret_name) => secret_name,
        None => {
            eprintln!("Usage: {} <secret-name>", args[0]);
            std::process::exit(1);
        }
    };

    println!("Fetching details for secret: {}", secret_name);

    let config = kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions::default()).await?;

    // aws::external_secret::list(config).await?;

    let secret = secrets::secret::get(config.clone(), secret_name).await?;

    let external_secret =
        secrets::external_secret::get(config.clone(), &secrets::secret::get_owner(&secret)).await?;

    let cluster_secret_store = secrets::cluster_secret_store::get(
        config,
        &external_secret.spec.secret_store_ref.unwrap().name,
    )
    .await?;

    println!(
        "{}",
        serde_yaml::to_string(&cluster_secret_store.spec).unwrap()
    );

    let provider_name = match &cluster_secret_store.spec.provider.kind {
        secrets::cluster_secret_store::ProviderType::Aws(_) => "aws",
        secrets::cluster_secret_store::ProviderType::Gcp(_) => "gcp",
        secrets::cluster_secret_store::ProviderType::Oracle(oracle) => {
            let provider = providers::oracle::OracleProvider::new();
            println!("{:?}", provider.get_identity().get_current_user().await);

            // let vatul_ocid = "ocid1.vault.oc1.eu-frankfurt-1.entqnjjeaafoa.abtheljr7gcl5vu75z5kxvmm4nwbgr4wpgh5uvsgzlvhzkq4wabywkre446a";
            // let secret_name = String::from_str("loki").unwrap();
            let secret_name = external_secret.spec.data_from[0].extract.key.as_str();

            let result = provider.get_secret(&secret_name, &oracle.vault).await?;
            let secret = result.text().await?;
            println!("{:?}", secret);
            "oracle"
        }
    };
    println!("🔍 Secret Store Provider: {}", provider_name);

    // 2. access provider and get secrets according to external_secret.spec.data_from
    // 3. get secret keys from secret.data
    // 4. figure out from which secret in merge list comes each key value
    // 5. output secret.data as YAML annotated with secret store path from which it comes

    Ok(())
}
