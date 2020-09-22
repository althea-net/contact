use crate::client::Contact;
use crate::jsonrpc::error::JsonRpcError;
use crate::types::*;
use deep_space::address::Address;

impl Contact {
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

    /// Get the latest valset recorded by the peggy module. If no valset has ever been created
    /// you will instead get a blank valset at height 0. Any value above this may or may not
    /// be a complete valset and it's up to the caller to interpret the response.
    pub async fn get_peggy_valset(&self) -> Result<ResponseWrapper<Valset>, JsonRpcError> {
        let none: Option<bool> = None;
        let ret: Result<ResponseWrapper<ValsetUnparsed>, JsonRpcError> = self
            .jsonrpc_client
            .request_method("peggy/current_valset", none, self.timeout, None)
            .await;
        match ret {
            Ok(val) => Ok(ResponseWrapper {
                height: val.height,
                result: val.result.convert(),
            }),
            Err(e) => Err(e),
        }
    }

    /// get the valset for a given nonce (block) height
    pub async fn get_peggy_valset_request(
        &self,
        nonce: u128,
    ) -> Result<ResponseWrapper<Valset>, JsonRpcError> {
        let none: Option<bool> = None;
        let ret: Result<ResponseWrapper<TypeWrapper<ValsetUnparsed>>, JsonRpcError> = self
            .jsonrpc_client
            .request_method(
                &format!("peggy/valset_request/{}", nonce),
                none,
                self.timeout,
                None,
            )
            .await;
        match ret {
            Ok(val) => Ok(ResponseWrapper {
                height: val.height,
                result: val.result.value.convert(),
            }),
            Err(e) => Err(e),
        }
    }

    /// This hits the /pending_valset_requests endpoint and will provide the oldest
    /// validator set we have not yet signed.
    pub async fn get_oldest_unsigned_valset(
        &self,
        address: Address,
    ) -> Result<ResponseWrapper<Valset>, JsonRpcError> {
        let none: Option<bool> = None;
        let ret: Result<ResponseWrapper<TypeWrapper<ValsetUnparsed>>, JsonRpcError> = self
            .jsonrpc_client
            .request_method(
                &format!("peggy/pending_valset_requests/{}", address),
                none,
                self.timeout,
                None,
            )
            .await;
        match ret {
            Ok(val) => Ok(ResponseWrapper {
                height: val.height,
                result: val.result.value.convert(),
            }),
            Err(e) => Err(e),
        }
    }

    /// this input views the last five valest requests that have been made, useful if you're
    /// a relayer looking to ferry confirmations
    pub async fn get_last_valset_requests(
        &self,
    ) -> Result<ResponseWrapper<Vec<Valset>>, JsonRpcError> {
        let none: Option<bool> = None;
        let ret: Result<ResponseWrapper<Vec<ValsetUnparsed>>, JsonRpcError> = self
            .jsonrpc_client
            .request_method(
                &"peggy/valset_requests".to_string(),
                none,
                self.timeout,
                None,
            )
            .await;

        match ret {
            Ok(val) => {
                let mut converted_values = Vec::new();
                for item in val.result {
                    converted_values.push(item.convert());
                }
                Ok(ResponseWrapper {
                    height: val.height,
                    result: converted_values,
                })
            }
            Err(e) => Err(e),
        }
    }

    /// get all valset confirmations for a given nonce
    pub async fn get_all_valset_confirms(
        &self,
        nonce: u64,
    ) -> Result<ResponseWrapper<Vec<ValsetConfirmResponse>>, JsonRpcError> {
        let none: Option<bool> = None;
        let ret: Result<ResponseWrapper<Vec<ValsetConfirmResponse>>, JsonRpcError> = self
            .jsonrpc_client
            .request_method(
                &format!("peggy/valset_confirm/{}", nonce),
                none,
                self.timeout,
                None,
            )
            .await;
        match ret {
            Ok(val) => Ok(val),
            Err(e) => Err(e),
        }
    }
}
