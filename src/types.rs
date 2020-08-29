use clarity::Address as EthAddress;
use deep_space::address::Address;
use deep_space::coin::Coin;
use num256::Uint256;
use serde::de::Deserializer;
use serde::{de, Deserialize};
use serde_json::Value;
use std::{fmt::Display, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CosmosAccountInfoWrapper {
    #[serde(deserialize_with = "parse_val")]
    pub height: u128,
    pub result: CosmosAccountWrapper,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CosmosAccountWrapper {
    #[serde(rename = "type")]
    pub account_type: String,
    pub value: CosmosAccountInfo,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CosmosAccountInfo {
    pub address: String,
    pub public_key: String,
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
    #[serde(rename = "Validator")]
    pub validator: Option<Address>,
    #[serde(rename = "Nonce")]
    pub nonce: Uint256,
    #[serde(rename = "Signature")]
    pub eth_signature: Option<Vec<u8>>,
}

/// wrapper struct for the valset response endpoint which
/// returns the response wrapped in this struct containing
/// info about which block the response is in reference to
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ValsetResponseWrapper {
    pub height: u128,
    pub result: ValsetResponse,
}

/// a list of validators, powers, and eth addresses at a given block height
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ValsetResponse {
    pub nonce: Uint256,
    pub powers: Vec<Uint256>,
    pub eth_addresses: Vec<Option<EthAddress>>,
}

/// a list of validators, powers, and eth addresses at a given block height
/// this version is used by the endpoint to get the data and is then processed
/// by "convert" into ValsetResponse. Making this struct purely internal
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ValsetResponseUnparsed {
    #[serde(rename = "Nonce", deserialize_with = "parse_val")]
    nonce: Uint256,
    #[serde(rename = "Powers")]
    powers: Vec<Uint256>,
    #[serde(rename = "EthAddresses")]
    eth_addresses: Vec<String>,
}
/// Another internal intermediary function to dedal with empty
/// strings as addresses
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ValsetResponseUnparsedWrapper {
    #[serde(deserialize_with = "parse_val")]
    pub height: u128,
    pub result: ValsetResponseUnparsed,
}

impl ValsetResponseUnparsedWrapper {
    pub fn convert(self) -> ValsetResponseWrapper {
        ValsetResponseWrapper {
            height: self.height,
            result: self.result.convert(),
        }
    }
}

impl ValsetResponseUnparsed {
    pub fn convert(self) -> ValsetResponse {
        let mut out = Vec::new();
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
        ValsetResponse {
            nonce: self.nonce,
            powers: self.powers,
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

        let _decoded: CosmosAccountInfoWrapper = serde_json::from_str(&file).unwrap();
    }
}
