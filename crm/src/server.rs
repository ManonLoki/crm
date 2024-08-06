use anyhow::Result;
use crm::pb::{
    user_service_server::{UserService, UserServiceServer},
    CreateUserRequest, GetUserRequest, User,
};
use tonic::{async_trait, transport::Server, Request, Response, Status};

/// 创建一个数据结构 用于存储许需要的数据
#[derive(Default)]
pub struct UserServer {}

// 实现RPC定义
#[async_trait]
impl UserService for UserServer {
    // 获取User
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("get_user: {:?}", input);
        Ok(Response::new(User::default()))
    }
    // 创建User
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("create_user: {:?}", input);
        let user = User::new(1, &input.name, &input.email);
        Ok(Response::new(user))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 监听端口
    let addr = "[::1]:50051".parse().unwrap();
    // 创建Server实例
    let svc = UserServer::default();

    println!("UserService listening on {}", addr);
    // 构建Server 这里的UserServiceServer是RPC生成的，将实例传入进去即可
    Server::builder()
        .add_service(UserServiceServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
