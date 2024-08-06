use anyhow::Result;
use crm::pb::{user_service_client::UserServiceClient, CreateUserRequest};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect到监听地址
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;

    // 创建调用Request
    let request = Request::new(CreateUserRequest {
        name: "ManonLoki".to_string(),
        email: "manonloki@gmail.com".to_string(),
    });

    // 获取响应
    let response = client.create_user(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
