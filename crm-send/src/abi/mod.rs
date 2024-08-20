mod email;
mod in_app;
mod sms;
use std::{ops::Deref, sync::Arc, time::Duration};

use chrono::Utc;
use futures::{Stream, StreamExt};
use prost_types::Timestamp;
use tokio::{sync::mpsc, time::sleep};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use tracing::info;

use crate::{
    config::AppConfig,
    pb::{notification_server::NotificationServer, send_request::Msg, SendRequest, SendResponse},
    NotificationService, NotificationServiceInner, ResponseStream, ServiceResult,
};

const CHANNEL_SIZE: usize = 1024;

#[allow(async_fn_in_trait)]
pub trait Sender {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status>;
}

impl NotificationService {
    pub fn new(config: AppConfig) -> Self {
        let inner = Arc::new(NotificationServiceInner {
            config,
            sender: dummy_send(),
        });
        NotificationService { inner }
    }

    pub fn into_server(self) -> NotificationServer<Self> {
        NotificationServer::new(self)
    }

    pub async fn send(
        &self,
        mut stream: impl Stream<Item = Result<SendRequest, Status>> + Send + 'static + Unpin,
    ) -> ServiceResult<ResponseStream> {
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
        let notif = self.clone();

        tokio::spawn(async move {
            while let Some(Ok(req)) = stream.next().await {
                let notif_clone = notif.clone();
                let res = match req.msg {
                    Some(Msg::Sms(sms)) => sms.send(notif_clone).await,
                    Some(Msg::Email(email)) => email.send(notif_clone).await,
                    Some(Msg::InApp(in_app)) => in_app.send(notif_clone).await,
                    None => Err(Status::invalid_argument("Invalid message type")),
                };

                tx.send(res).await.unwrap();
            }
        });

        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream)))
    }
}

impl Deref for NotificationService {
    type Target = NotificationServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn dummy_send() -> mpsc::Sender<Msg> {
    let (tx, mut rx) = mpsc::channel(CHANNEL_SIZE * 100);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            info!("Sending message:{:?}", msg);
            sleep(Duration::from_millis(300)).await;
        }
    });

    tx
}

fn to_ts() -> Timestamp {
    let now = Utc::now();
    Timestamp {
        seconds: now.timestamp(),
        nanos: now.timestamp_subsec_nanos() as i32,
    }
}
