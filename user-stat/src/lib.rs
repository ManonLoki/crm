pub mod abi;
pub mod config;
mod pb;

use config::AppConfig;
use futures::Stream;
use pb::{
    user_stats_server::{UserStats, UserStatsServer},
    QueryRequest, RawQueryRequest, User,
};
use sqlx::PgPool;
use std::{ops::Deref, pin::Pin, sync::Arc};
use tonic::{async_trait, Request, Response, Status};

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send>>;

/// UserStatsService
#[derive(Clone)]
pub struct UserStatsService {
    inner: Arc<UserStatsServiceInner>,
}

/// 内部数据，此数据通过Arc方式Clone
#[allow(dead_code)]
pub struct UserStatsServiceInner {
    config: AppConfig,
    pool: PgPool,
}

/// 为UserStats实现UserStats RPC Trait
#[async_trait]
impl UserStats for UserStatsService {
    // 实现QueryStream
    type QueryStream = ResponseStream;
    // 实现RawQueryStream
    type RawQueryStream = ResponseStream;

    // Query
    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        let query = request.into_inner();
        self.query(query).await
    }
    // RawQuert
    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        let query = request.into_inner();
        self.raw_query(query).await
    }
}

impl UserStatsService {
    // 实现创建一个新的Service实例
    pub async fn new(config: AppConfig) -> Self {
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .expect("Failed Connect to DB");
        let inner = UserStatsServiceInner { config, pool };

        Self {
            inner: Arc::new(inner),
        }
    }

    // 将Service转换为 RPC Server
    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
