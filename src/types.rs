use clarity::Address as EthAddress;
use clarity::Signature as EthSignature;
use deep_space::address::Address;
use deep_space::coin::Coin;
use deep_space::public_key::PublicKey;
use num256::Uint256;
use serde::de::Deserializer;
use serde::{de, Deserialize};
use serde_json::Value;
use std::{fmt::Display, str::FromStr};

/// A generic wrapper for Cosmos REST server responses which always
/// include the height
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseWrapper<T> {
    #[serde(deserialize_with = "parse_val")]
    pub height: u128,
    pub result: T,
}

/// A generic wrapper for Cosmos REST server responses which always
/// include the struct type
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TypeWrapper<T> {
    #[serde(rename = "type")]
    pub struct_type: String,
    pub value: T,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CosmosAccountInfo {
    #[serde(deserialize_with = "parse_val_option")]
    pub address: Option<Address>,
    #[serde(deserialize_with = "parse_val_option")]
    pub public_key: Option<PublicKey>,
    pub account_number: u128,
    pub coins: Vec<Coin>,
    pub sequence: u128,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BlockId {
    hash: String,
    parts: BlockParts,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BlockParts {
    pub total: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BlockHeader {
    pub version: BlockVersion,
    pub chain_id: String,
    pub time: String,
    pub last_block_id: BlockId,
    pub last_commit_hash: String,
    pub data_hash: String,
    pub validators_hash: String,
    pub next_validators_hash: String,
    pub consensus_hash: String,
    pub app_hash: String,
    pub last_results_hash: String,
    pub evidence_hash: String,
    #[serde(deserialize_with = "parse_val")]
    pub proposer_address: Address,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BlockVersion {
    #[serde(deserialize_with = "parse_val")]
    pub block: u128,
    #[serde(deserialize_with = "parse_val")]
    pub app: u128,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LatestBlockEndpointResponse {
    pub block_id: BlockId,
    pub block: Block,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub data: BlockData,
    pub evidence: BlockEvidence,
    pub last_commit: LastCommit,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BlockData {
    pub txs: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BlockEvidence {
    pub evidence: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LastCommit {
    #[serde(deserialize_with = "parse_val")]
    height: u128,
    #[serde(deserialize_with = "parse_val")]
    round: u128,
    block_id: BlockId,
    signatures: Vec<BlockSignature>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BlockSignature {
    pub block_id_flag: u128,
    #[serde(deserialize_with = "parse_val")]
    pub validator_address: Address,
    pub timestamp: String,
    pub signature: String,
}

/// the response we get when querying for a valset confirmation
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ValsetConfirmResponse {
    #[serde(deserialize_with = "parse_val")]
    pub validator: Address,
    #[serde(deserialize_with = "parse_val")]
    pub nonce: Uint256,
    #[serde(deserialize_with = "parse_val", rename = "signature")]
    pub eth_signature: EthSignature,
}

/// a list of validators, powers, and eth addresses at a given block height
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Valset {
    pub nonce: u64,
    pub powers: Vec<u64>,
    pub eth_addresses: Vec<Option<EthAddress>>,
}

/// a list of validators, powers, and eth addresses at a given block height
/// this version is used by the endpoint to get the data and is then processed
/// by "convert" into ValsetResponse. Making this struct purely internal
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ValsetUnparsed {
    #[serde(deserialize_with = "parse_val")]
    nonce: u64,
    powers: Vec<String>,
    eth_addresses: Vec<String>,
}

impl ValsetUnparsed {
    pub fn convert(self) -> Valset {
        let mut out = Vec::new();
        let mut powers = Vec::new();
        for maybe_addr in self.eth_addresses.iter() {
            if maybe_addr.is_empty() {
                out.push(None);
            } else {
                match maybe_addr.parse() {
                    Ok(val) => out.push(Some(val)),
                    Err(_e) => out.push(None),
                }
            }
        }
        for power in self.powers.iter() {
            match power.parse() {
                Ok(val) => powers.push(val),
                Err(_e) => powers.push(0),
            }
        }
        Valset {
            nonce: self.nonce,
            powers,
            eth_addresses: out,
        }
    }
}

/// the query struct required to get the valset request sent by a specific
/// validator. This is required because the url encoded get methods don't
/// parse addresses well. So there's no way to get an individual validators
/// address without sending over a json body
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct QueryValsetConfirm {
    pub nonce: String,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct OptionalTXInfo {
    pub chain_id: String,
    pub account_number: u128,
    pub sequence: u128,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TXSendResponse {
    #[serde(deserialize_with = "parse_val_option", default)]
    pub gas_used: Option<u128>,
    #[serde(deserialize_with = "parse_val_option", default)]
    pub gas_wanted: Option<u128>,
    #[serde(deserialize_with = "parse_val")]
    pub height: u128,
    pub logs: Option<Value>,
    #[serde(default)]
    pub raw_log: Value,
    pub txhash: String,
}

/// Adapter that lets us parse any val that implements from_str into
/// the type we want, this helps solve type problems like sigs or addresses
/// being presented as strings and requiring a parse. For our own types like
/// Address we just implement deserialize such that the string representation
/// is accepted implicitly. But for native types like u128 this is the only
/// way to go
fn parse_val<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

fn parse_val_option<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    match T::from_str(&s) {
        Ok(val) => Ok(Some(val)),
        Err(_e) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn decode_block_summary() {
        let file =
            read_to_string("test_files/block_endpoint.json").expect("Failed to read test files!");
        let _decoded: LatestBlockEndpointResponse = serde_json::from_str(&file).unwrap();

        let file =
            read_to_string("test_files/block_endpoint_2.json").expect("Failed to read test files!");

        let _decoded: LatestBlockEndpointResponse = serde_json::from_str(&file).unwrap();
    }

    #[test]
    fn decode_account_info() {
        let file =
            read_to_string("test_files/account_info.json").expect("Failed to read test files!");

        let _decoded: ResponseWrapper<CosmosAccountInfo> = serde_json::from_str(&file).unwrap();
    }
}
