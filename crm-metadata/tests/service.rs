use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use crm_metadata::{
    config::AppConfig,
    pb::{metadata_client::MetadataClient, MaterializeRequest},
    MetadataService,
};
use futures::StreamExt;
use tokio::time::sleep;
use tonic::{transport::Server, Request};

#[tokio::test]
async fn test_metadata() -> Result<()> {
    let addr = start_server().await?;
    // 创建GRPC Client
    let mut client = MetadataClient::connect(format!("http://{}", addr)).await?;
    // 创建请求Stream
    let stream = tokio_stream::iter(vec![
        MaterializeRequest { id: 1 },
        MaterializeRequest { id: 2 },
        MaterializeRequest { id: 3 },
    ]);
    // 包装为Tonic Stream
    let request = Request::new(stream);
    // 拿到Response Stream
    let response = client.materialize(request).await?.into_inner();
    // 转换为Vec<Content>
    let ret: Vec<_> = response
        .then(|res| async move { res.unwrap() })
        .collect()
        .await;

    assert_eq!(ret.len(), 3);
    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    let config = AppConfig::load()?;
    let addr = format!("[::1]:{}", config.server.port).parse()?;

    let svc = MetadataService::new(config).into_server();
    tokio::spawn(async move {
        Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .unwrap();
    });

    sleep(Duration::from_micros(1)).await;

    Ok(addr)
}
