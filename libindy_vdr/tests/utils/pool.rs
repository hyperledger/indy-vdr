use indy_test_utils::genesis::GenesisTransactions;
use indy_vdr::common::error::VdrResult;
use indy_vdr::ledger::RequestBuilder;
use indy_vdr::pool::helpers::{perform_ledger_action, perform_ledger_request};
use indy_vdr::pool::{
    NodeReplies, Pool, PoolBuilder, PoolTransactions, PreparedRequest, RequestResult, SharedPool,
};

use futures_executor::block_on;

pub struct TestPool {
    pool: SharedPool,
}

impl TestPool {
    pub fn new() -> TestPool {
        let pool_transactions =
            PoolTransactions::from_json_transactions(GenesisTransactions::default_transactions())
                .unwrap();

        let pool = PoolBuilder::default()
            .transactions(pool_transactions)
            .unwrap()
            .into_shared()
            .unwrap();

        TestPool { pool }
    }

    pub fn transactions(&self) -> Vec<String> {
        self.pool.get_json_transactions().unwrap()
    }

    pub fn request_builder(&self) -> RequestBuilder {
        self.pool.get_request_builder()
    }

    pub fn send_request(&self, prepared_request: &PreparedRequest) -> Result<String, String> {
        block_on(async {
            let (request_result, _timing) = perform_ledger_request(&self.pool, prepared_request)
                .await
                .unwrap();

            match request_result {
                RequestResult::Reply(message) => Ok(message),
                RequestResult::Failed(err) => Err(err.extra().unwrap_or_default()),
            }
        })
    }

    pub fn send_full_request(
        &self,
        prepared_request: &PreparedRequest,
        node_aliases: Option<Vec<String>>,
        timeout: Option<i64>,
    ) -> VdrResult<NodeReplies<String>> {
        block_on(async {
            let (request_result, _timing) = perform_ledger_action(
                &self.pool,
                prepared_request.req_id.clone(),
                prepared_request.req_json.to_string(),
                node_aliases,
                timeout,
            )
            .await
            .unwrap();
            match request_result {
                RequestResult::Reply(replies) => Ok(replies),
                RequestResult::Failed(err) => Err(err),
            }
        })
    }

    pub fn send_request_with_retries(
        &self,
        prepared_request: &PreparedRequest,
        previous_response: &str,
    ) -> Result<String, String> {
        Self::_submit_retry(
            Self::extract_seq_no_from_reply(previous_response).unwrap(),
            || self.send_request(prepared_request),
        )
    }

    pub fn extract_seq_no_from_reply(reply: &str) -> Result<u64, &'static str> {
        let reply: serde_json::Value =
            serde_json::from_str(reply).map_err(|_| "Cannot deserialize transaction Response")?;

        let seq_no = reply["result"]["seqNo"]
            .as_u64()
            .or_else(|| reply["result"]["txnMetadata"]["seqNo"].as_u64())
            .ok_or("Missed seqNo in reply")?;

        Ok(seq_no)
    }

    const SUBMIT_RETRY_CNT: usize = 3;

    fn _submit_retry<F>(minimal_timestamp: u64, submit_action: F) -> Result<String, String>
    where
        F: Fn() -> Result<String, String>,
    {
        let mut i = 0;
        let action_result = loop {
            let action_result = submit_action()?;

            let retry = Self::extract_seq_no_from_reply(&action_result)
                .map(|received_timestamp| received_timestamp < minimal_timestamp)
                .unwrap_or(true);

            if retry && i < Self::SUBMIT_RETRY_CNT {
                ::std::thread::sleep(::std::time::Duration::from_secs(2));
                i += 1;
            } else {
                break action_result;
            }
        };
        Ok(action_result)
    }
}
