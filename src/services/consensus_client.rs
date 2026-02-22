use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::service_client::ServiceClient as TmServiceClient;
use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::{
    GetBlockByHeightRequest, GetBlockByHeightResponse, GetLatestBlockRequest,
    GetLatestBlockResponse,
};
use cosmos_sdk_proto::cosmos::tx::v1beta1::service_client::ServiceClient as TxServiceClient;
use cosmos_sdk_proto::cosmos::tx::v1beta1::{
    BroadcastMode, BroadcastTxRequest, BroadcastTxResponse,
};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tonic::transport::Channel;
use tonic::transport::Endpoint;

#[derive(Clone)]
pub struct ConsensusClient {
    pub tx_client: TxServiceClient<Channel>,
    pub tm_client: TmServiceClient<Channel>,
    pub healthy: Arc<AtomicBool>,
}

impl ConsensusClient {
    pub async fn connect(
        endpoint: String,
        healthy: Arc<AtomicBool>,
    ) -> Result<Self, tonic::transport::Error> {
        // 确保端点字符串包含 http 前缀；Endpoint::from_shared 一般会处理该情况
        // Cosmos gRPC 通常运行在 HTTP/2 协议之上
        let channel = Endpoint::from_shared(endpoint)?
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await?;

        let tx_client =
            TxServiceClient::new(channel.clone()).max_decoding_message_size(16 * 1024 * 1024);
        let tm_client = TmServiceClient::new(channel).max_decoding_message_size(16 * 1024 * 1024);

        Ok(Self {
            tx_client,
            tm_client,
            healthy,
        })
    }

    pub async fn broadcast_tx(
        &mut self,
        tx_bytes: Vec<u8>,
    ) -> Result<BroadcastTxResponse, tonic::Status> {
        let mut retries = 0;
        let max_retries = 3;
        let mut backoff = Duration::from_millis(500);

        loop {
            let req = BroadcastTxRequest {
                tx_bytes: tx_bytes.clone(),
                mode: BroadcastMode::Sync as i32,
            };

            match self.tx_client.broadcast_tx(req).await {
                Ok(resp) => {
                    self.healthy.store(true, Ordering::SeqCst);
                    return Ok(resp.into_inner());
                }
                Err(status) => {
                    if (status.code() == tonic::Code::Unavailable
                        || status.code() == tonic::Code::Unknown)
                        && retries < max_retries
                    {
                        retries += 1;
                        sleep(backoff).await;
                        backoff *= 2;
                        continue;
                    }
                    if retries >= max_retries {
                        self.healthy.store(false, Ordering::SeqCst);
                    }
                    return Err(status);
                }
            }
        }
    }

    pub async fn get_block(
        &mut self,
        height: i64,
    ) -> Result<GetBlockByHeightResponse, tonic::Status> {
        let req = GetBlockByHeightRequest { height };
        self.tm_client
            .get_block_by_height(req)
            .await
            .map(|r| r.into_inner())
    }

    pub async fn get_latest_block(&mut self) -> Result<GetLatestBlockResponse, tonic::Status> {
        let req = GetLatestBlockRequest {};
        self.tm_client
            .get_latest_block(req)
            .await
            .map(|r| r.into_inner())
    }
}
