use crate::jsonrpc::client::HTTPClient;
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
    pub jsonrpc_client: Arc<Box<HTTPClient>>,
    pub timeout: Duration,
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

    /// If you run the start-chains.sh script in the peggy repo it will pass
    /// port 1317 on localhost through to the peggycli rest-server which can
    /// then be used to run this test and debug things quickly. You will need
    /// to run the following command and copy a phrase so that you actually
    /// have some coins to send funds
    /// docker exec -it peggy_test_instance cat /validator-phrases
    #[test]
    #[ignore]
    fn test_endpoints() {
        let key = PrivateKey::from_phrase("speed drastic talent solution divert cheap caution road dance sign empty aisle gift kangaroo inherit build fury general cup skirt staff present sponsor marriage", "").unwrap();
        let contact = Contact::new("http://localhost:1317", Duration::from_secs(30));
        let token_name = "footoken".to_string();

        let res = System::run(move || {
            Arbiter::spawn(async move {
                let res = test_rpc_calls(contact, key, token_name).await;
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
