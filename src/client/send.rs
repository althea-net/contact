use crate::client::Contact;
use crate::jsonrpc::error::JsonRpcError;
use crate::types::*;
use crate::utils::maybe_get_optional_tx_info;
use clarity::Address as EthAddress;
use clarity::{abi::encode_tokens, abi::Token, PrivateKey as EthPrivateKey};
use deep_space::address::Address;
use deep_space::coin::Coin;
use deep_space::msg::{Msg, SendMsg, SetEthAddressMsg, ValsetConfirmMsg, ValsetRequestMsg};
use deep_space::private_key::PrivateKey;
use deep_space::stdfee::StdFee;
use deep_space::stdsignmsg::StdSignMsg;
use deep_space::transaction::Transaction;
use deep_space::transaction::TransactionSendType;
use deep_space::utils::bytes_to_hex_str;

impl Contact {
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
                gas: 500_000u64.into(),
            },
            msgs: vec![Msg::SendMsg(SendMsg {
                from_address: our_address,
                to_address: destination,
                amount: vec![coin],
            })],
            memo: String::new(),
        };

        let tx = private_key
            .sign_std_msg(std_sign_msg, TransactionSendType::Block)
            .unwrap();
        trace!("{}", json!(tx));

        self.jsonrpc_client
            .request_method("txs", Some(tx), self.timeout, None)
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
        trace!(
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
                gas: 500_000u64.into(),
            },
            msgs: vec![Msg::SetEthAddressMsg(SetEthAddressMsg {
                eth_address,
                validator: our_address,
                eth_signature: bytes_to_hex_str(&eth_signature.to_bytes()),
            })],
            memo: String::new(),
        };

        let tx = private_key
            .sign_std_msg(std_sign_msg, TransactionSendType::Block)
            .unwrap();

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
                gas: 500_000u64.into(),
            },
            msgs: vec![Msg::ValsetRequestMsg(ValsetRequestMsg {
                requester: our_address,
            })],
            memo: String::new(),
        };

        let tx = private_key
            .sign_std_msg(std_sign_msg, TransactionSendType::Block)
            .unwrap();
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
        valset: Valset,
        private_key: PrivateKey,
        peggy_id: String,
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

        let message = encode_tokens(&[
            Token::FixedString(peggy_id),
            Token::FixedString("checkpoint".to_string()),
            valset.nonce.into(),
            normalize_addresses(&valset.eth_addresses).into(),
            valset.powers.into(),
        ]);
        let eth_signature = eth_private_key.sign_msg(&message);

        let std_sign_msg = StdSignMsg {
            chain_id: tx_info.chain_id,
            account_number: tx_info.account_number,
            sequence: tx_info.sequence,
            fee: StdFee {
                amount: vec![fee],
                gas: 500_000u64.into(),
            },
            msgs: vec![Msg::ValsetConfirmMsg(ValsetConfirmMsg {
                validator: our_address,
                nonce: valset.nonce.into(),
                eth_signature: bytes_to_hex_str(&eth_signature.to_bytes()),
            })],
            memo: String::new(),
        };

        let tx = private_key
            .sign_std_msg(std_sign_msg, TransactionSendType::Block)
            .unwrap();

        self.jsonrpc_client
            .request_method("txs", Some(tx), self.timeout, None)
            .await
    }
}

/// Takes an array of Option<EthAddress> and converts to EthAddress by replacing None
/// values with a zero address
fn normalize_addresses(input: &[Option<EthAddress>]) -> Vec<EthAddress> {
    let mut output = Vec::new();
    for val in input.iter() {
        match val {
            Some(a) => output.push(*a),
            None => output.push(EthAddress::from_slice(&[0; 20]).unwrap()),
        }
    }
    output
}
