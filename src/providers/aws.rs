use k8s_openapi::api::core::v1::Secret;

#[derive(Clone)]
pub struct AWSProvider {}

impl AWSProvider {
    pub fn new() -> Self {
        AWSProvider {}
    }

    pub fn handle(&self, secret: Secret) {
        match secret.data {
            Some(data) => {
                for (key, value) in data {
                    let decoded_value = String::from_utf8(value.0).unwrap();
                    println!("{}: {}", key, decoded_value);
                }
            }
            None => println!("No data found in secret"),
        }
    }

    // pub async fn get_secret(
    //     &self,
    //     secret_name: &str,
    //     vault_ocid: &str,
    // ) -> Result<Response, Box<dyn std::error::Error>> {

    // }
}
