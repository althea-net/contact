use crate::jsonrpc::error::JsonRpcError;
use crate::jsonrpc::request::Request;
use crate::jsonrpc::response::Response;
use actix_web::client::Client;
use actix_web::http::header;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::str;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct HTTPClient {
    id_counter: Arc<Mutex<RefCell<i64>>>,
    url: String,
    client: Client,
}

impl HTTPClient {
    pub fn new(url: &str) -> Self {
        Self {
            id_counter: Arc::new(Mutex::new(RefCell::new(0i64))),
            url: url.to_string(),
            client: Client::default(),
        }
    }

    fn next_id(&self) -> i64 {
        let counter = self.id_counter.clone();
        let counter = counter.lock().expect("id error");
        let mut value = counter.borrow_mut();
        *value += 1;
        *value
    }

    pub async fn request_method<T: Serialize, R: 'static>(
        &self,
        method: &str,
        params: T,
        timeout: Duration,
        request_size_limit: Option<usize>,
    ) -> Result<R, JsonRpcError>
    where
        for<'de> R: Deserialize<'de>,
        // T: std::fmt::Debug,
        R: std::fmt::Debug,
    {
        // the payload size limit for this request, almost everything
        // will set this to None, and get the default 64k, but some requests
        // need bigger buffers (like full block requests)
        let limit = match request_size_limit {
            Some(val) => val,
            None => 65536,
        };
        let payload = Request::new(self.next_id(), method, params);
        let res = self
            .client
            .post(&self.url)
            .header(header::CONTENT_TYPE, "application/json")
            .timeout(timeout)
            .send_json(&payload)
            .await;
        let mut res = match res {
            Ok(val) => val,
            Err(e) => return Err(JsonRpcError::FailedToSend(e.to_string())),
        };
        println!("{:?}", res);
        let res: Response<R> = match res.json().limit(limit).await {
            Ok(val) => val,
            Err(e) => return Err(JsonRpcError::BadResponse(e.to_string())),
        };
        trace!("got Cosmos JSONRPC response {:#?}", res);
        println!("got Cosmos JSONRPC response {:#?}", res);
        let data = res.data.into_result();

        match data {
            Ok(val) => Ok(val),
            Err(e) => Err(JsonRpcError::ResponseError {
                code: e.code,
                message: e.message,
                data: format!("{:?}", e.data),
            }),
        }
    }
}
