pub mod error;

use error::VdrProxyClientError;
use reqwest::{Client, Response, Url};

pub use indy_vdr::ledger::RequestBuilder;
pub use indy_vdr::pool::PreparedRequest;

pub struct VdrProxyClient {
    client: Client,
    url: Url,
}

async fn map_resp(response: Response) -> Result<String, VdrProxyClientError> {
    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(VdrProxyClientError::NonSuccessStatusCode(
            status.as_u16(),
            text,
        ));
    }
    response
        .text()
        .await
        .map_err(VdrProxyClientError::HttpClientError)
}

impl VdrProxyClient {
    pub fn new(url: &str) -> Result<VdrProxyClient, VdrProxyClientError> {
        let url = Url::parse(url)?;
        let client = Client::new();
        Ok(VdrProxyClient { client, url })
    }

    async fn get_request(&self, url: Url) -> Result<String, VdrProxyClientError> {
        let response = self.client.get(url).send().await?;
        map_resp(response).await
    }

    async fn post_request(
        &self,
        url: Url,
        request: PreparedRequest,
    ) -> Result<String, VdrProxyClientError> {
        let response = self
            .client
            .post(url)
            .json(&request.req_json)
            .send()
            .await
            .map_err(VdrProxyClientError::HttpClientError)?;
        map_resp(response).await
    }

    pub async fn post(&self, request: PreparedRequest) -> Result<String, VdrProxyClientError> {
        let url = self.url.join("submit")?;
        self.post_request(url, request).await
    }

    pub async fn get_nym(&self, did: &str) -> Result<String, VdrProxyClientError> {
        let url = self.url.join(&format!("nym/{}", did))?;
        self.get_request(url).await
    }

    pub async fn get_attrib(&self, did: &str, attrib: &str) -> Result<String, VdrProxyClientError> {
        let url = self.url.join(&format!("attrib/{}/{}", did, attrib))?;
        self.get_request(url).await
    }

    pub async fn get_schema(&self, schema_id: &str) -> Result<String, VdrProxyClientError> {
        let url = self.url.join(&format!("schema/{}", schema_id))?;
        self.get_request(url).await
    }

    pub async fn get_cred_def(&self, cred_def_id: &str) -> Result<String, VdrProxyClientError> {
        let url = self.url.join(&format!("cred_def/{}", cred_def_id))?;
        self.get_request(url).await
    }

    pub async fn get_rev_reg(&self, rev_reg_def_id: &str) -> Result<String, VdrProxyClientError> {
        let url = self.url.join(&format!("rev_reg/{}", rev_reg_def_id))?;
        self.get_request(url).await
    }

    pub async fn get_rev_reg_def(
        &self,
        rev_reg_def_id: &str,
    ) -> Result<String, VdrProxyClientError> {
        let url = self.url.join(&format!("rev_reg_def/{}", rev_reg_def_id))?;
        self.get_request(url).await
    }

    pub async fn get_rev_reg_delta(
        &self,
        rev_reg_def_id: &str,
    ) -> Result<String, VdrProxyClientError> {
        let url = self
            .url
            .join(&format!("rev_reg_delta/{}", rev_reg_def_id))?;
        self.get_request(url).await
    }

    pub async fn get_txn_author_agreement(&self) -> Result<String, VdrProxyClientError> {
        let url = self.url.join("taa")?;
        self.get_request(url).await
    }

    pub async fn get_genesis_txs(&self) -> Result<String, VdrProxyClientError> {
        let url = self.url.join("genesis")?;
        self.get_request(url).await
    }

    pub async fn get_acceptance_methods_list(&self) -> Result<String, VdrProxyClientError> {
        let url = self.url.join("aml")?;
        self.get_request(url).await
    }

    pub async fn get_auth_rules(&self) -> Result<String, VdrProxyClientError> {
        let url = self.url.join("auth")?;
        self.get_request(url).await
    }

    pub async fn get_proxy_status(&self) -> Result<String, VdrProxyClientError> {
        self.get_request(self.url.clone()).await
    }

    pub async fn get_ledger_txn(
        &self,
        subledger: &str,
        seq_no: u64,
    ) -> Result<String, VdrProxyClientError> {
        let url = self.url.join(&format!("txn/{}/{}", subledger, seq_no))?;
        self.get_request(url).await
    }
}
