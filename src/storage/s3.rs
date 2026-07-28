use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::Client;

pub async fn create_s3_client() -> Client {
    let mut config_loader = aws_config::defaults(BehaviorVersion::latest());

    if let Ok(region) = std::env::var("AWS_REGION") {
        config_loader = config_loader.region(Region::new(region));
    }

    let sdk_config = config_loader.load().await;

    let mut s3_config = aws_sdk_s3::config::Builder::from(&sdk_config);

    if let Ok(endpoint_url) = std::env::var("AWS_ENDPOINT_URL") {
        s3_config = s3_config.endpoint_url(endpoint_url);
    }

    if let Ok(force_path_style) = std::env::var("S3_FORCE_PATH_STYLE") {
        let enabled = matches!(force_path_style.as_str(), "1" | "true" | "TRUE" | "True");
        s3_config = s3_config.force_path_style(enabled);
    }

    Client::from_conf(s3_config.build())
}
