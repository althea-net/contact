use crate::client::Contact;
use crate::jsonrpc::error::JsonRpcError;
use crate::types::*;
use deep_space::address::Address;

impl Contact {
    pub async fn get_latest_number(&self) -> Result<u128, JsonRpcError> {
        let none: Option<bool> = None;
        let res: Result<LatestBlockEndpointResponse, JsonRpcError> = self
            .jsonrpc_client
            .request_method("blocks/latest", none, self.timeout, None)
            .await;

        match res {
            Ok(res) => Ok(res.block.last_commit.height),
            Err(e) => Err(e),
        }
    }

    pub async fn get_latest_block(&self) -> Result<LatestBlockEndpointResponse, JsonRpcError> {
        let none: Option<bool> = None;
        self.jsonrpc_client
            .request_method("blocks/latest", none, self.timeout, None)
            .await
    }

    /// Gets account info for the provided Cosmos account using the accounts endpoint
    pub async fn get_account_info(
        &self,
        address: Address,
    ) -> Result<ResponseWrapper<TypeWrapper<CosmosAccountInfo>>, JsonRpcError> {
        let none: Option<bool> = None;
        self.jsonrpc_client
            .request_method(
                &format!("auth/accounts/{}", address),
                none,
                self.timeout,
                None,
            )
            .await
    }
}
