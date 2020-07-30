use crate::jsonrpc::client::HTTPClient;
use crate::jsonrpc::error::JsonRpcError;
use crate::types::LatestBlockEndpointResponse;
use std::sync::Arc;
use std::time::Duration;

/// An instance of Contact Cosmos RPC Client.
#[derive(Clone)]
pub struct Contact {
    jsonrpc_client: Arc<Box<HTTPClient>>,
    timeout: Duration,
}

impl Contact {
    pub fn new(url: &str, timeout: Duration) -> Self {
        Self {
            jsonrpc_client: Arc::new(Box::new(HTTPClient::new(url))),
            timeout,
        }
    }

    pub async fn get_latest_block(&self) -> Result<LatestBlockEndpointResponse, JsonRpcError> {
        self.jsonrpc_client
            .request_method("block", Vec::<String>::new(), self.timeout, None)
            .await
    }
}
