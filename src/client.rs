use crate::jsonrpc::client::HTTPClient;
use crate::jsonrpc::error::JsonRpcError;
use crate::types::*;
use deep_space::address::Address;
use num256::Uint256;
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
        let none: Option<bool> = None;
        self.jsonrpc_client
            .request_method("blocks/latest", none, self.timeout, None)
            .await
    }

    /// Get the latest valset recorded by the peggy module may or may not be complete
    pub async fn get_peggy_valset(&self) -> Result<ValsetResponse, JsonRpcError> {
        let none: Option<bool> = None;
        self.jsonrpc_client
            .request_method("peggy/current_valset", none, self.timeout, None)
            .await
    }

    /// get the valset for a given nonce (block) height
    pub async fn get_peggy_valset_request(
        &self,
        nonce: Uint256,
    ) -> Result<ValsetResponse, JsonRpcError> {
        let none: Option<bool> = None;
        self.jsonrpc_client
            .request_method(
                &format!("peggy/valset_request/{}", nonce),
                none,
                self.timeout,
                None,
            )
            .await
    }

    /// get specific confirmations for a given valset, this is useful
    /// when ferrying valsets over to the Cosmos chain
    pub async fn get_peggy_valset_confirmation(
        &self,
        nonce: Uint256,
        validator_address: Address,
    ) -> Result<ValsetConfirmResponse, JsonRpcError> {
        let none: Option<bool> = None;
        self.jsonrpc_client
            .request_method(
                &format!("peggy/valset_confirm/{}/{}", nonce, validator_address),
                none,
                self.timeout,
                None,
            )
            .await
    }

    /// Send a transaction updating the eth address for the sending
    /// Cosmos address. The sending Cosmos address should be a validator
    pub async fn update_peggy_eth_address(
        &self,
        msg: UpdateEthAddressTX,
    ) -> Result<(), JsonRpcError> {
        self.jsonrpc_client
            .request_method("peggy/update_ethaddr", Some(msg), self.timeout, None)
            .await
    }

    /// Send a transaction requesting that a valset be formed for a given block
    /// height
    pub async fn send_valset_request(&self, msg: ValsetRequestTX) -> Result<(), JsonRpcError> {
        self.jsonrpc_client
            .request_method("peggy/valset_request", Some(msg), self.timeout, None)
            .await
    }

    /// Send in a confirmation for a specific validator set for a specific block height
    pub async fn send_valset_confirm(&self, msg: ValsetConfirmTX) -> Result<(), JsonRpcError> {
        self.jsonrpc_client
            .request_method("peggy/valset_confirm", Some(msg), self.timeout, None)
            .await
    }
}
