use chrono::{DateTime, TimeZone, Utc};
use itertools::Itertools;
use prost_types::Timestamp;
use tonic::{Response, Status};

use crate::{
    pb::{QueryRequest, RawQueryRequest, User},
    ResponseStream, ServiceResult, UserStatsService,
};

// 实现UserStatsService内部函数
impl UserStatsService {
    // 条件查询
    pub async fn query(&self, query: QueryRequest) -> ServiceResult<ResponseStream> {
        // 组成头部
        let mut sql = "SELECT email,name FROM user_stats WHERE ".to_string();

        // 组时间查询条件
        let time_conditions = query
            .timestamps
            .into_iter()
            .map(|(k, v)| timestamp_query(&k, v.lower, v.upper))
            .join(" AND ");

        sql.push_str(&time_conditions);

        // 组ID查询条件
        let id_conditions = query
            .ids
            .into_iter()
            .map(|(k, v)| ids_query(&k, v.ids))
            .join(" AND ");

        sql.push_str(" AND ");
        sql.push_str(&id_conditions);

        println!("Generated SQL: {}", sql);
        // 调用raw_query
        self.raw_query(RawQueryRequest { query: sql }).await
    }

    // 原始SQL查询
    pub async fn raw_query(&self, req: RawQueryRequest) -> ServiceResult<ResponseStream> {
        // SQLX 拿到列表
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.inner.pool)
            .await
        else {
            return Err(Status::internal(format!(
                "Failed to fetch data with query:{}",
                req.query
            )));
        };

        // 将Iterator 转换为Stream
        Ok(Response::new(Box::pin(futures::stream::iter(
            ret.into_iter().map(Ok),
        ))))
    }
}

// 组ID条件
fn ids_query(name: &str, ids: Vec<u32>) -> String {
    if ids.is_empty() {
        return "TRUE".to_string();
    }

    format!("array{:?} <@ {}", ids, name)
}
// 组时间戳条件
fn timestamp_query(name: &str, lower: Option<Timestamp>, upper: Option<Timestamp>) -> String {
    if lower.is_none() && upper.is_none() {
        return "TRUE".to_string();
    }

    if lower.is_none() {
        let upper = ts_to_utc(upper.unwrap());
        return format!("{} <= '{}'", name, upper.to_rfc3339());
    }

    if upper.is_none() {
        let lower = ts_to_utc(lower.unwrap());
        return format!("{} >= '{}'", name, lower.to_rfc3339());
    }

    format!(
        "{} BETWEEN '{}' AND {}",
        name,
        ts_to_utc(upper.unwrap()).to_rfc3339(),
        ts_to_utc(lower.unwrap()).to_rfc3339()
    )
}

// 将时间戳 转换为UTC时间
fn ts_to_utc(ts: Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as _).unwrap()
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use futures::StreamExt;

    use crate::{
        pb::{IdQuery, QueryRequestBuilder, TimeQuery},
        AppConfig,
    };

    use super::*;

    #[tokio::test]
    async fn raw_query_should_work() -> Result<()> {
        let config = AppConfig::load().expect("Failed Load config");
        let svc = UserStatsService::new(config).await;
        let mut stream = svc
            .raw_query(RawQueryRequest {
                query: "select * from user_stats where created_at > '2024-01-01' limit 5"
                    .to_string(),
            })
            .await?
            .into_inner();

        while let Some(res) = stream.next().await {
            println!("{:?}", res);
        }

        Ok(())
    }

    #[tokio::test]
    async fn query_should_work() -> Result<()> {
        let config = AppConfig::load().expect("Failed Load config");
        let svc = UserStatsService::new(config).await;
        let query = QueryRequestBuilder::default()
            .timestamp(("created_at".to_string(), tq(Some(120), None)))
            .timestamp(("last_visited_at".to_string(), tq(Some(30), None)))
            .id(("viewed_but_not_started".to_string(), id(&[252790])))
            .build()
            .unwrap();

        let mut stream = svc.query(query).await?.into_inner();

        while let Some(res) = stream.next().await {
            println!("{:?}", res);
        }
        Ok(())
    }

    fn id(id: &[u32]) -> IdQuery {
        IdQuery { ids: id.to_vec() }
    }
    fn tq(lower: Option<i64>, upper: Option<i64>) -> TimeQuery {
        TimeQuery {
            lower: lower.map(to_ts),
            upper: upper.map(to_ts),
        }
    }

    fn to_ts(days: i64) -> Timestamp {
        let dt = Utc::now()
            .checked_add_signed(chrono::Duration::days(days))
            .unwrap();

        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
}
