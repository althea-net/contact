use crate::types::*;
use crate::{client::Contact, error::CosmosGrpcError};
use bytes::BytesMut;
use cosmos_sdk_proto::cosmos::auth::v1beta1::{
    query_client::QueryClient as AuthQueryClient, ModuleAccount, QueryAccountRequest,
};
use cosmos_sdk_proto::cosmos::bank::v1beta1::query_client::QueryClient as BankQueryClient;
use cosmos_sdk_proto::cosmos::bank::v1beta1::QueryAllBalancesRequest;
use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::service_client::ServiceClient as TendermintServiceClient;
use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::GetLatestBlockRequest;
use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::GetSyncingRequest;
use cosmos_sdk_proto::cosmos::tx::v1beta1::service_client::ServiceClient as TxServiceClient;
use cosmos_sdk_proto::cosmos::tx::v1beta1::GetTxRequest;
use cosmos_sdk_proto::cosmos::tx::v1beta1::GetTxResponse;
use deep_space::address::Address;
use prost::Message;

impl Contact {
    /// Gets the current chain status, returns an enum taking into account the various possible states
    /// of the chain and the requesting full node. In the common case this provides the block number
    pub async fn get_chain_status(&self) -> Result<ChainStatus, CosmosGrpcError> {
        let mut grpc = TendermintServiceClient::connect(self.url.clone()).await?;
        let syncing = grpc.get_syncing(GetSyncingRequest {}).await?.into_inner();

        if syncing.syncing {
            Ok(ChainStatus::Syncing)
        } else {
            let block = grpc.get_latest_block(GetLatestBlockRequest {}).await?;
            let block = block.into_inner().block;
            match block {
                Some(block) => match block.last_commit {
                    // for some reason the block height can be negative, we cast it to a u64 for the sake
                    // of logical bounds checking
                    Some(commit) => Ok(ChainStatus::Moving {
                        block_height: commit.height as u64,
                    }),
                    None => Err(CosmosGrpcError::BadResponse(
                        "No commit in block?".to_string(),
                    )),
                },
                None => Ok(ChainStatus::WaitingToStart),
            }
        }
    }

    /// Gets the latest block from the node, taking into account the possibility that the chain is halted
    /// and also the possibility that the node is syncing
    pub async fn get_latest_block(&self) -> Result<LatestBlock, CosmosGrpcError> {
        let mut grpc = TendermintServiceClient::connect(self.url.clone()).await?;
        let syncing = grpc
            .get_syncing(GetSyncingRequest {})
            .await?
            .into_inner()
            .syncing;

        let block = grpc.get_latest_block(GetLatestBlockRequest {}).await?;
        let block = block.into_inner().block;
        match block {
            Some(block) => {
                if syncing {
                    Ok(LatestBlock::Syncing { block })
                } else {
                    Ok(LatestBlock::Latest { block })
                }
            }
            None => Ok(LatestBlock::WaitingToStart),
        }
    }

    /// Gets account info for the provided Cosmos account using the accounts endpoint
    /// accounts do not have any info if they have no tokens or are otherwise never seen
    /// before an Ok(None) result indicates this
    pub async fn get_account_info(&self, address: Address) -> Result<BaseAccount, CosmosGrpcError> {
        let mut agrpc = AuthQueryClient::connect(self.url.clone()).await?;
        let res = agrpc
            // todo detect chain prefix here
            .account(QueryAccountRequest {
                address: address.to_string(),
            })
            .await?
            .into_inner();
        let account = res.account;
        match account {
            Some(value) => {
                let mut buf = BytesMut::with_capacity(value.value.len());
                buf.copy_from_slice(&value.value);
                let decoded: ModuleAccount = Message::decode(buf)?;
                match decoded.base_account {
                    Some(b) => Ok(b.into()),
                    None => Err(CosmosGrpcError::NoToken),
                }
            }
            None => Err(CosmosGrpcError::NoToken),
        }
    }

    // Gets a transaction using it's hash value, TODO should fail if the transaction isn't found
    pub async fn get_tx_by_hash(&self, txhash: String) -> Result<GetTxResponse, CosmosGrpcError> {
        let mut txrpc = TxServiceClient::connect(self.url.clone()).await?;
        let res = txrpc
            .get_tx(GetTxRequest { hash: txhash })
            .await?
            .into_inner();
        Ok(res)
    }

    pub async fn get_balances(&self, address: Address) -> Result<Vec<Coin>, CosmosGrpcError> {
        let mut bankrpc = BankQueryClient::connect(self.url.clone()).await?;
        let res = bankrpc
            .all_balances(QueryAllBalancesRequest {
                // TODO determine chain prefix and make sure we're using that prefix
                address: address.to_string(),
                pagination: None,
            })
            .await?
            .into_inner();
        let balances = res.balances;
        let mut ret = Vec::new();
        for value in balances {
            ret.push(value.into());
        }
        Ok(ret)
    }
}
