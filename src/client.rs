use crate::jsonrpc::client::HTTPClient;
use crate::jsonrpc::error::JsonRpcError;
use crate::types::*;
use crate::utils::maybe_get_optional_tx_info;
use clarity::PrivateKey as EthPrivateKey;
use deep_space::address::Address;
use deep_space::coin::Coin;
use deep_space::msg::{Msg, SendMsg};
use deep_space::private_key::PrivateKey;
use deep_space::stdfee::StdFee;
use deep_space::stdsignmsg::BaseReq;
use deep_space::stdsignmsg::StdSignMsg;
use deep_space::transaction::Transaction;
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

    /// Gets account info for the provided Cosmos account using the accounts endpoint
    pub async fn get_account_info(
        &self,
        address: Address,
    ) -> Result<CosmosAccountInfoWrapper, JsonRpcError> {
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

    /// The advanced version of create_and_send transaction that expects you to
    /// perform your own signing and prep first.
    pub async fn send_transaction(&self, msg: Transaction) -> Result<(), JsonRpcError> {
        self.jsonrpc_client
            .request_method("txs", Some(msg), self.timeout, None)
            .await
    }

    /// The hand holding version of send transaction that does it all for you
    pub async fn create_and_send_transaction(
        &self,
        coin: Coin,
        destination: Address,
        private_key: PrivateKey,
        chain_id: Option<String>,
        account_number: Option<u64>,
        sequence: Option<u64>,
    ) -> Result<(), JsonRpcError> {
        let our_address = private_key
            .to_public_key()
            .expect("Invalid private key!")
            .to_address();

        let tx_info =
            maybe_get_optional_tx_info(our_address, chain_id, account_number, sequence, &self)
                .await?;

        // todo there is no way to estimate gas, fix in
        // Cosmos
        let std_sign_msg = StdSignMsg {
            chain_id: tx_info.chain_id,
            account_number: tx_info.account_number,
            sequence: tx_info.sequence,
            fee: StdFee {
                amount: vec![Coin {
                    denom: coin.denom.clone(),
                    amount: 42u32.into(),
                }],
                gas: 200_000u64.into(),
            },
            msgs: vec![Msg::SendMsg(SendMsg {
                from_address: our_address,
                to_address: destination,
                amount: vec![coin],
            })],
            memo: String::new(),
        };

        let tx = private_key.sign_std_msg(std_sign_msg).unwrap();

        self.jsonrpc_client
            .request_method("blocks/latest", Some(tx), self.timeout, None)
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
        eth_private_key: EthPrivateKey,
        private_key: PrivateKey,
        chain_id: Option<String>,
        account_number: Option<u64>,
        sequence: Option<u64>,
    ) -> Result<(), JsonRpcError> {
        let our_address = private_key
            .to_public_key()
            .expect("Invalid private key!")
            .to_address();

        let tx_info =
            maybe_get_optional_tx_info(our_address, chain_id, account_number, sequence, &self)
                .await?;

        // todo there is no way to estimate gas, fix in
        // Cosmos
        let base_request = BaseReq {
            chain_id: tx_info.chain_id,
            account_number: tx_info.account_number,
            sequence: tx_info.sequence,
            fee: StdFee {
                // todo figure out native coin denom and drop it in maybe_get_optional_tx_info
                amount: vec![Coin {
                    denom: "denom".to_string(),
                    amount: 42u32.into(),
                }],
                gas: 200_000u64.into(),
            },
            msgs: Vec::new(),
            memo: String::new(),
        };

        let eth_address = eth_private_key.to_public_key().unwrap();
        let eth_signature = eth_private_key.sign_msg(eth_address.to_string().as_bytes());

        let msg: UpdateEthAddressTX = UpdateEthAddressTX {
            base_request,
            eth_signature: eth_signature.to_bytes().to_vec(),
        };

        self.jsonrpc_client
            .request_method("peggy/update_ethaddr", Some(msg), self.timeout, None)
            .await
    }

    /// Send a transaction requesting that a valset be formed for a given block
    /// height
    pub async fn send_valset_request(
        &self,
        private_key: PrivateKey,
        chain_id: Option<String>,
        account_number: Option<u64>,
        sequence: Option<u64>,
    ) -> Result<(), JsonRpcError> {
        let our_address = private_key
            .to_public_key()
            .expect("Invalid private key!")
            .to_address();

        let tx_info =
            maybe_get_optional_tx_info(our_address, chain_id, account_number, sequence, &self)
                .await?;

        // todo there is no way to estimate gas, fix in
        // Cosmos
        let base_request = BaseReq {
            chain_id: tx_info.chain_id,
            account_number: tx_info.account_number,
            sequence: tx_info.sequence,
            fee: StdFee {
                // todo figure out native coin denom and drop it in maybe_get_optional_tx_info
                amount: vec![Coin {
                    denom: "denom".to_string(),
                    amount: 42u32.into(),
                }],
                gas: 200_000u64.into(),
            },
            msgs: Vec::new(),
            memo: String::new(),
        };

        let msg = ValsetRequestTX { base_request };

        self.jsonrpc_client
            .request_method("peggy/valset_request", Some(msg), self.timeout, None)
            .await
    }

    /// Send in a confirmation for a specific validator set for a specific block height
    #[allow(clippy::too_many_arguments)]
    pub async fn send_valset_confirm(
        &self,
        eth_private_key: EthPrivateKey,
        valset: Vec<u8>,
        valset_nonce: Uint256,
        private_key: PrivateKey,
        chain_id: Option<String>,
        account_number: Option<u64>,
        sequence: Option<u64>,
    ) -> Result<(), JsonRpcError> {
        let our_address = private_key
            .to_public_key()
            .expect("Invalid private key!")
            .to_address();

        let tx_info =
            maybe_get_optional_tx_info(our_address, chain_id, account_number, sequence, &self)
                .await?;

        // todo there is no way to estimate gas, fix in
        // Cosmos
        let base_request = BaseReq {
            chain_id: tx_info.chain_id,
            account_number: tx_info.account_number,
            sequence: tx_info.sequence,
            fee: StdFee {
                // todo figure out native coin denom and drop it in maybe_get_optional_tx_info
                amount: vec![Coin {
                    denom: "denom".to_string(),
                    amount: 42u32.into(),
                }],
                gas: 200_000u64.into(),
            },
            msgs: Vec::new(),
            memo: String::new(),
        };

        let eth_signature = eth_private_key.sign_msg(&valset);

        let msg = ValsetConfirmTX {
            base_request,
            nonce: valset_nonce,
            eth_signature: eth_signature.to_bytes().to_vec(),
        };

        self.jsonrpc_client
            .request_method("peggy/valset_confirm", Some(msg), self.timeout, None)
            .await
    }
}
