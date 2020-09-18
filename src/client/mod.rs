use crate::jsonrpc::client::HTTPClient;
use clarity::PrivateKey as EthPrivateKey;
use deep_space::address::Address;
use deep_space::coin::Coin;
use deep_space::private_key::PrivateKey;
use std::sync::Arc;
use std::time::Duration;

mod get;
mod send;

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
    /// then be used to run this test and debug things quickly. You will need
    /// to run the following command and copy a phrase so that you actually
    /// have some coins to send funds
    /// docker exec -it peggy_test_instance cat /validator-phrases
    #[test]
    #[ignore]
    fn test_endpoints() {
        let mut rng = rand::thread_rng();
        let secret: [u8; 32] = rng.gen();

        let key = PrivateKey::from_phrase("deal eternal voice label table flight raw pear bless glove marine letter paddle fringe modify just carbon soda maid hybrid chronic patch phone mixture", "").unwrap();
        let eth_private_key = EthPrivateKey::from_slice(&secret).expect("Failed to parse eth key");
        let contact = Contact::new("http://localhost:1317", Duration::from_secs(30));
        let token_name = "footoken".to_string();

        let res = System::run(move || {
            Arbiter::spawn(async move {
                let res = test_rpc_calls(contact, key, eth_private_key, token_name).await;
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
    test_token_name: String,
) -> Result<(), String> {
    let fee = Coin {
        denom: test_token_name.clone(),
        amount: 1u32.into(),
    };
    let address = key
        .to_public_key()
        .expect("Failed to convert to pubkey!")
        .to_address();

    test_basic_calls(&contact, key, test_token_name, fee.clone(), address).await?;
    // set eth address also tested here, TODO expand to include things like changing
    // the set eth address
    test_valset_request_calls(&contact, key, eth_private_key, fee.clone()).await?;
    //test_valset_confirm_calls(&contact, key, eth_private_key, fee.clone()).await?;

    Ok(())
}

async fn test_basic_calls(
    contact: &Contact,
    key: PrivateKey,
    test_token_name: String,
    fee: Coin,
    address: Address,
) -> Result<(), String> {
    // start by validating the basics
    //
    // get the latest block
    // get our account info
    // send a base transaction

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
                denom: test_token_name.clone(),
                amount: 5u32.into(),
            },
            fee.clone(),
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
    Ok(())
}

async fn test_valset_request_calls(
    contact: &Contact,
    key: PrivateKey,
    eth_private_key: EthPrivateKey,
    fee: Coin,
) -> Result<(), String> {
    // next we update our eth address so that we can be sure it's present in the resulting valset
    // request
    let res = contact
        .update_peggy_eth_address(eth_private_key, key, fee.clone(), None, None, None)
        .await;
    if res.is_err() {
        return Err(format!("Failed to update eth address {:?}", res));
    }

    let res = contact.get_peggy_valset_request(1).await;
    if res.is_ok() {
        return Err(format!(
            "Got valset request that should not exist {:?}",
            res
        ));
    }

    // we request a valset be created
    // and then look at results at two block heights, one where the request was made, one where it
    // was not
    let res = contact
        .send_valset_request(key, fee.clone(), None, None, None)
        .await;
    if res.is_err() {
        return Err(format!("Failed to create valset request {:?}", res));
    }
    let valset_request_block = res.unwrap().height;

    let res = contact.get_peggy_valset_request(valset_request_block).await;
    println!("valset response is {:?}", res);
    if let Ok(valset) = res {
        assert_eq!(valset.height, valset_request_block);

        let addresses = valset.result.eth_addresses;
        if !addresses.contains(&Some(eth_private_key.to_public_key().unwrap())) {
            // we successfully submitted our eth address before, we should find it now
            return Err("Incorrect Valset, does not include submitted eth address".to_string());
        }
    } else {
        return Err("Failed to get valset request that should exist".to_string());
    }
    let res = contact.get_peggy_valset_request(valset_request_block).await;
    println!("valset response is {:?}", res);
    if let Ok(valset) = res {
        // this is actually a timing issue, but should be true
        assert_eq!(valset.height, valset_request_block);

        let addresses = valset.result.eth_addresses.clone();
        if !addresses.contains(&Some(eth_private_key.to_public_key().unwrap())) {
            // we successfully submitted our eth address before, we should find it now
            return Err("Incorrect Valset, does not include submitted eth address".to_string());
        }

        println!("Sending valset confirm!");
        // issue here, we can't actually test valset confirm because all the validators need
        // to have submitted an Ethereum address first.
        let res = contact
            .send_valset_confirm(
                eth_private_key,
                fee,
                valset.result,
                key,
                "test".to_string(),
                None,
                None,
                None,
            )
            .await;
        if res.is_err() {
            return Err(format!("Failed to send valset confirm {:?}", res));
        }
    } else {
        return Err("Failed to get valset request that should exist".to_string());
    }

    // valset confirm

    Ok(())
}
