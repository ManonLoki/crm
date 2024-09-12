use anyhow::Result;

use crm::{
    pb::{crm_client::CrmClient, WelcomeRequestBuilder},
    AppConfig,
};
use tonic::Request;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;
    let mut client = CrmClient::connect(format!("http://[::1]:{}", config.server.port)).await?;

    let req = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(93u32)
        .content_ids([1u32, 2, 3])
        .build()?;

    let response = client.welcome(Request::new(req)).await?.into_inner();
    println!("RESPONSE={:?}", response);
    Ok(())
}
