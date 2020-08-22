use crate::jsonrpc::client::HTTPClient;
use crate::jsonrpc::error::JsonRpcError;
use crate::types::*;
use crate::utils::maybe_get_optional_tx_info;
use clarity::PrivateKey as EthPrivateKey;
use deep_space::address::Address;
use deep_space::coin::Coin;
use deep_space::msg::{Msg, SendMsg, SetEthAddressMsg, ValsetConfirmMsg, ValsetRequestMsg};
use deep_space::private_key::PrivateKey;
use deep_space::stdfee::StdFee;
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
        let mut url = url;
        if !url.ends_with('/') {
            url = url.trim_end_matches('/');
        }
        Self {
            jsonrpc_client: Arc::new(Box::new(HTTPClient::new(&url))),
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
    pub async fn send_transaction(&self, msg: Transaction) -> Result<TXSendResponse, JsonRpcError> {
        self.jsonrpc_client
            .request_method("txs", Some(msg), self.timeout, None)
            .await
    }

    /// The hand holding version of send transaction that does it all for you
    #[allow(clippy::too_many_arguments)]
    pub async fn create_and_send_transaction(
        &self,
        coin: Coin,
        fee: Coin,
        destination: Address,
        private_key: PrivateKey,
        chain_id: Option<String>,
        account_number: Option<u128>,
        sequence: Option<u128>,
    ) -> Result<TXSendResponse, JsonRpcError> {
        trace!("Creating transaction");
        let our_address = private_key
            .to_public_key()
            .expect("Invalid private key!")
            .to_address();

        let tx_info =
            maybe_get_optional_tx_info(our_address, chain_id, account_number, sequence, &self)
                .await?;

        let std_sign_msg = StdSignMsg {
            chain_id: tx_info.chain_id,
            account_number: tx_info.account_number,
            sequence: tx_info.sequence,
            fee: StdFee {
                amount: vec![fee],
                gas: 20_000u64.into(),
            },
            msgs: vec![Msg::SendMsg(SendMsg {
                from_address: our_address,
                to_address: destination,
                amount: vec![coin],
            })],
            memo: String::new(),
        };

        let tx = private_key.sign_std_msg(std_sign_msg).unwrap();
        trace!("{}", json!(tx));

        self.jsonrpc_client
            .request_method("txs", Some(tx), self.timeout, None)
            .await
    }

    /// Get the latest valset recorded by the peggy module may or may not be complete
    pub async fn get_peggy_valset(&self) -> Result<ValsetResponseWrapper, JsonRpcError> {
        let none: Option<bool> = None;
        self.jsonrpc_client
            .request_method("peggy/current_valset", none, self.timeout, None)
            .await
    }

    /// get the valset for a given nonce (block) height
    pub async fn get_peggy_valset_request(
        &self,
        nonce: u128,
    ) -> Result<ValsetResponseWrapper, JsonRpcError> {
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
        nonce: u128,
        validator_address: Address,
    ) -> Result<ValsetConfirmResponse, JsonRpcError> {
        let payload = QueryValsetConfirm {
            nonce: nonce.to_string(),
            address: format!("{}", validator_address),
        };
        self.jsonrpc_client
            .request_method(
                &"peggy/query_valset_confirm".to_string(),
                Some(payload),
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
        fee: Coin,
        chain_id: Option<String>,
        account_number: Option<u128>,
        sequence: Option<u128>,
    ) -> Result<TXSendResponse, JsonRpcError> {
        trace!("Updating Peggy ETH address");
        let our_address = private_key
            .to_public_key()
            .expect("Invalid private key!")
            .to_address();

        let tx_info =
            maybe_get_optional_tx_info(our_address, chain_id, account_number, sequence, &self)
                .await?;
        trace!("got optional tx info");

        let eth_address = eth_private_key.to_public_key().unwrap();
        let eth_signature = eth_private_key.sign_msg(our_address.as_bytes());
        println!(
            "sig: {} address: {}",
            clarity::utils::bytes_to_hex_str(&eth_signature.to_bytes()),
            clarity::utils::bytes_to_hex_str(eth_address.as_bytes())
        );

        let std_sign_msg = StdSignMsg {
            chain_id: tx_info.chain_id,
            account_number: tx_info.account_number,
            sequence: tx_info.sequence,
            fee: StdFee {
                amount: vec![fee],
                gas: 20_000u64.into(),
            },
            msgs: vec![Msg::SetEthAddressMsg(SetEthAddressMsg {
                eth_address,
                validator: our_address,
                eth_signature: eth_signature.to_bytes().to_vec(),
            })],
            memo: String::new(),
        };

        let tx = private_key.sign_std_msg(std_sign_msg).unwrap();

        self.jsonrpc_client
            .request_method("txs", Some(tx), self.timeout, None)
            .await
    }

    /// Send a transaction requesting that a valset be formed for a given block
    /// height
    pub async fn send_valset_request(
        &self,
        private_key: PrivateKey,
        fee: Coin,
        chain_id: Option<String>,
        account_number: Option<u128>,
        sequence: Option<u128>,
    ) -> Result<TXSendResponse, JsonRpcError> {
        let our_address = private_key
            .to_public_key()
            .expect("Invalid private key!")
            .to_address();

        let tx_info =
            maybe_get_optional_tx_info(our_address, chain_id, account_number, sequence, &self)
                .await?;

        let std_sign_msg = StdSignMsg {
            chain_id: tx_info.chain_id,
            account_number: tx_info.account_number,
            sequence: tx_info.sequence,
            fee: StdFee {
                amount: vec![fee],
                gas: 20_000u64.into(),
            },
            msgs: vec![Msg::ValsetRequestMsg(ValsetRequestMsg {
                requester: our_address,
            })],
            memo: String::new(),
        };

        let tx = private_key.sign_std_msg(std_sign_msg).unwrap();
        trace!("{}", json!(tx));

        self.jsonrpc_client
            .request_method("txs", Some(tx), self.timeout, None)
            .await
    }

    /// Send in a confirmation for a specific validator set for a specific block height
    #[allow(clippy::too_many_arguments)]
    pub async fn send_valset_confirm(
        &self,
        eth_private_key: EthPrivateKey,
        fee: Coin,
        valset: Vec<u8>,
        valset_nonce: Uint256,
        private_key: PrivateKey,
        chain_id: Option<String>,
        account_number: Option<u128>,
        sequence: Option<u128>,
    ) -> Result<TXSendResponse, JsonRpcError> {
        let our_address = private_key
            .to_public_key()
            .expect("Invalid private key!")
            .to_address();

        let tx_info =
            maybe_get_optional_tx_info(our_address, chain_id, account_number, sequence, &self)
                .await?;

        let eth_signature = eth_private_key.sign_msg(&valset);

        // todo determine what this operation costs and use that rather than 42
        let std_sign_msg = StdSignMsg {
            chain_id: tx_info.chain_id,
            account_number: tx_info.account_number,
            sequence: tx_info.sequence,
            fee: StdFee {
                amount: vec![fee],
                gas: 20_000u64.into(),
            },
            msgs: vec![Msg::ValsetConfirmMsg(ValsetConfirmMsg {
                validator: our_address,
                nonce: valset_nonce,
                eth_signature: eth_signature.to_bytes().to_vec(),
            })],
            memo: String::new(),
        };

        let tx = private_key.sign_std_msg(std_sign_msg).unwrap();

        self.jsonrpc_client
            .request_method("peggy/valset_confirm", Some(tx), self.timeout, None)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix::Arbiter;
    use actix::System;
    use rand::{self, Rng};
    /// simple test used to get raw signature bytes to feed into other
    /// applications for testing. Specifically to get signing compatibility
    /// with go-ethereum
    #[test]
    #[ignore]
    fn get_sig() {
        use sha3::{Digest, Keccak256};
        let mut rng = rand::thread_rng();
        let secret: [u8; 32] = rng.gen();
        let eth_private_key = EthPrivateKey::from_slice(&secret).expect("Failed to parse eth key");
        let eth_address = eth_private_key.to_public_key().unwrap();
        let msg = eth_address.as_bytes();
        let eth_signature = eth_private_key.sign_msg(msg);
        let digest = Keccak256::digest(msg);
        println!(
            "sig: 0x{} hash: 0x{} address: 0x{}",
            clarity::utils::bytes_to_hex_str(&eth_signature.to_bytes()),
            clarity::utils::bytes_to_hex_str(&digest),
            clarity::utils::bytes_to_hex_str(eth_address.as_bytes())
        );
    }

    /// If you run the start-chains.sh script in the peggy repo it will pass
    /// port 1317 on localhost through to the peggycli rest-server which can
    /// then be used to run this test and debug things quickly. Obviously none
    /// of the transactions will actually send since the random address won't have
    /// any tokens. But the rpc server is kind enough to tell you when the tx would
    /// have sent and there just aren't funds.
    #[test]
    #[ignore]
    fn test_endpoints() {
        let mut rng = rand::thread_rng();
        let secret: [u8; 32] = rng.gen();

        let key = PrivateKey::from_secret(&secret);
        let eth_private_key = EthPrivateKey::from_slice(&secret).expect("Failed to parse eth key");
        let contact = Contact::new("http://localhost:1317", Duration::from_secs(5));

        let res = System::run(move || {
            Arbiter::spawn(async move {
                let res = test_rpc_calls(contact, key, eth_private_key, address).await;
                if res.is_err() {
                    println!("{:?}", res);
                    System::current().stop_with_code(1);
                }

                System::current().stop();
            });
        });

        if let Err(e) = res {
            panic!(format!("{:?}", e))
        }
    }
}

pub async fn test_rpc_calls(
    contact: Contact,
    key: PrivateKey,
    eth_private_key: EthPrivateKey,
) -> Result<(), String> {
    let address = key
        .to_public_key()
        .expect("Failed to convert to pubkey!")
        .to_address();

    let res = contact.get_latest_block().await;
    if res.is_err() {
        return Err(format!("Failed to get latest block {:?}", res));
    }

    let res = contact.get_account_info(address).await;
    if res.is_err() {
        return Err(format!("Failed to get account info {:?}", res));
    }

    let res = contact
        .create_and_send_transaction(
            Coin {
                denom: "test".to_string(),
                amount: 5u32.into(),
            },
            Coin {
                denom: "test".to_string(),
                amount: 5u32.into(),
            },
            key.to_public_key().unwrap().to_address(),
            key,
            None,
            None,
            None,
        )
        .await;
    if res.is_err() {
        return Err(format!("Failed to send tx {:?}", res));
    }

    let res = contact.get_peggy_valset_request(0).await;
    if res.is_err() {
        return Err(format!("Failed to get valset request {:?}", res));
    }

    let res = contact.get_peggy_valset().await;
    if res.is_err() {
        return Err(format!("Failed to get valset {:?}", res));
    }

    let res = contact.get_peggy_valset_confirmation(0, address).await;
    if res.is_err() {
        return Err(format!("Failed to get valset confirmation {:?}", res));
    }

    let res = contact
        .send_valset_request(
            key,
            Coin {
                denom: "test".to_string(),
                amount: 5u32.into(),
            },
            None,
            None,
            None,
        )
        .await;
    if res.is_err() {
        return Err(format!("Failed to send valset request {:?}", res));
    }

    let res = contact
        .update_peggy_eth_address(
            eth_private_key,
            key,
            Coin {
                denom: "test".to_string(),
                amount: 5u32.into(),
            },
            None,
            None,
            None,
        )
        .await;
    if res.is_err() {
        return Err(format!("Failed to update eth address {:?}", res));
    }
    Ok(())
}
