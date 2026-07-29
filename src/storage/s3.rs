use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::Client;

pub async fn create_s3_client() -> Client {
    let mut loader = aws_config::defaults(BehaviorVersion::latest());

    if let Ok(region) = std::env::var("AWS_REGION") {
        loader = loader.region(Region::new(region));
    }

    let sdk_config = loader.load().await;

    Client::new(&sdk_config)
}
