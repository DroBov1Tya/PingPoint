

pub mod request{
    use reqwest::Client;

    pub async fn req(webpath: String, endpoint: String) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();

        let response = client
        .get(&format!("{}{}", webpath, endpoint))
        .send()
        .await?;

        let status_code = response.status();
        let result_str: String = format!("{}{}: STATUS_CODE = {}", webpath, endpoint, status_code);

        Ok(result_str)
    }
    
}