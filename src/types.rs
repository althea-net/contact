use clarity::Address as EthAddress;
use deep_space::address::Address;
use deep_space::coin::Coin;
use deep_space::stdsignmsg::BaseReq;
use num256::Uint256;
use serde::{de, Deserialize, Deserializer};
use std::{fmt::Display, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CosmosAccountInfoWrapper {
    #[serde(rename = "type")]
    pub account_type: String,
    pub value: CosmosAccountInfo,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CosmosAccountInfo {
    pub account_number: String,
    pub coins: Vec<Coin>,
    pub sequence: String,
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
    pub txs: Option<Vec<Transaction>>,
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

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UpdateEthAddressTX {
    #[serde(rename = "BaseReq")]
    pub base_request: BaseReq,
    #[serde(rename = "EthSig")]
    pub eth_signature: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ValsetRequestTX {
    #[serde(rename = "BaseReq")]
    pub base_request: BaseReq,
}

/// a transaction we send to submit a valset confirmation signature
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ValsetConfirmTX {
    #[serde(rename = "BaseReq")]
    pub base_request: BaseReq,
    #[serde(rename = "Nonce")]
    pub nonce: Uint256,
    #[serde(rename = "EthSig")]
    pub eth_signature: Vec<u8>,
}

/// the response we get when querying for a valset confirmation
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ValsetConfirmResponse {
    #[serde(rename = "Validator")]
    pub validator: Address,
    #[serde(rename = "Nonce")]
    pub nonce: Uint256,
    #[serde(rename = "Signature")]
    pub eth_signature: Vec<u8>,
}

/// a list of validators, powers, and eth addresses at a given block height
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ValsetResponse {
    #[serde(rename = "Nonce")]
    pub nonce: Uint256,
    #[serde(rename = "Powers")]
    pub powers: Uint256,
    #[serde(rename = "EthAddresses")]
    pub eth_addresses: Vec<EthAddress>,
}

#[derive(Debug, Clone)]
pub struct OptionalTXInfo {
    pub chain_id: String,
    pub account_number: u64,
    pub sequence: u64,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn decode_block_summary() {
        let file =
            read_to_string("test_files/block_endpoint.json").expect("Failed to read test files!");

        let _decoded: LatestBlockEndpointResponse = serde_json::from_str(&file).unwrap();
    }
}
