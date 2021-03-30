use cosmos_sdk_proto::cosmos::auth::v1beta1::query_client::QueryClient as AuthQueryClient;
use cosmos_sdk_proto::cosmos::base::tendermint::v1beta1::service_client::ServiceClient as TendermintServiceClient;
use cosmos_sdk_proto::cosmos::staking::v1beta1::query_client::QueryClient as StakingQueryClient;
use cosmos_sdk_proto::cosmos::tx::v1beta1::service_client::ServiceClient as TxServiceClient;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Channel;

mod get;
mod send;

/// An instance of Contact Cosmos RPC Client.
#[derive(Clone)]
pub struct Contact {
    url: String,
    pub timeout: Duration,
}

impl Contact {
    pub fn new(url: &str, timeout: Duration) -> Self {
        let mut url = url;
        if !url.ends_with('/') {
            url = url.trim_end_matches('/');
        }
        Self {
            url: url.to_string(),
            timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deep_space::address::Address;
    use deep_space::coin::Coin;
    use deep_space::private_key::PrivateKey;
    use rand::Rng;

    /// If you run the start-chains.sh script in the Gravity repo it will pass
    /// port 26657 on localhost and allow you to debug things quickly
    /// then be used to run this test and debug things quickly. You will need
    /// to run the following command and copy a phrase so that you actually
    /// have some coins to send funds
    /// docker exec -it gravity_test_instance cat /validator-phrases
    #[test]
    #[ignore]
    fn test_endpoints() {
        env_logger::init();
        let key = PrivateKey::from_phrase("destroy lock crane champion nest hurt chicken leopard field album describe glimpse chimney sort kind peanut worry dilemma anchor dismiss fox there judge arm", "").unwrap();
        let token_name = "footoken".to_string();
    }
}
