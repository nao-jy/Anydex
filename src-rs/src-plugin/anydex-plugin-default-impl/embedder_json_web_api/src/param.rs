use jsonpath_rust::JsonPath;
use mime_guess::Mime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct APIParam {
    #[schemars(
        title = "Target Content Type",
        description = "Content type prefix to prcess with this API, see https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/MIME_types/Common_types"
    )]
    pub target_content_type_prefix: String,

    #[schemars(title = "Request Endpoint", url)]
    pub request_endpoint: String,

    #[schemars(title = "API Key")]
    pub api_key: Option<String>,

    #[schemars(
        title = "Request JSON",
        description = "JSON format for request. <text> <image_base64> are reserved and automatically injected"
    )]
    pub request_json: serde_json::Map<String, serde_json::Value>,

    #[schemars(
        title = "Response JSON Path",
        description = "Place of embedding in response, see https://jsonpath.com/"
    )]
    pub response_json_path: String,
}

impl APIParam {
    pub fn is_processable(&self, mime: &Mime) -> bool {
        mime.to_string()
            .to_lowercase()
            .starts_with(&self.target_content_type_prefix)
    }
    pub fn get_text_replaced(req_body: &str, text: &str) -> String {
        req_body.replace("<text>", text)
    }
    pub fn get_image_base64_replaced(req_body: &str, image_base64: &str) -> String {
        req_body.replace("<image_base64>", image_base64)
    }
    fn request_body(
        &self,
        text: Option<&str>,
        image_base64: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let mut body_json = serde_json::Value::from(self.request_json.clone()).to_string();
        if let Some(content) = text {
            let escaped_content = serde_json::to_string(content).map_err(|err| err.to_string())?;
            body_json = APIParam::get_text_replaced(
                &body_json,
                &escaped_content[1..escaped_content.len() - 1],
            );
        }
        if let Some(content) = image_base64 {
            let escaped_content = serde_json::to_string(content).map_err(|err| err.to_string())?;
            body_json = APIParam::get_image_base64_replaced(
                &body_json,
                &escaped_content[1..escaped_content.len() - 1],
            );
        }
        serde_json::from_str(&body_json).map_err(|err| err.to_string())
    }
    fn parse_embedding(response_text: &str, response_json_path: &str) -> Result<Vec<f32>, String> {
        let response_json: serde_json::Value = serde_json::from_str(response_text)
            .map_err(|err| format!("response is not valid JSON: {err}"))?;

        let result = response_json
            .query(response_json_path)
            .map_err(|err| err.to_string())?;
        let values = match result.as_slice() {
            [serde_json::Value::Array(values)] => values.iter().collect::<Vec<_>>(),
            values => values.to_vec(),
        };
        if values.is_empty() {
            return Err(format!("response is not as expected: {response_json}"));
        }

        values
            .into_iter()
            .map(|value| {
                value
                    .as_f64()
                    .map(|number| number as f32)
                    .ok_or_else(|| format!("response is not as expected: {response_json}"))
            })
            .collect()
    }
    pub fn try_get_embedding(
        &self,
        text: Option<String>,
        image_base64: Option<String>,
    ) -> Result<Vec<f32>, String> {
        let body_json = self.request_body(text.as_deref(), image_base64.as_deref())?;

        let client = reqwest::blocking::Client::new();
        let mut response = client.post(&self.request_endpoint).json(&body_json);
        if let Some(api_key) = &self.api_key {
            response = response.header("Authorization", format!("Bearer {api_key}"))
        }
        let response = response.send().map_err(|err| err.to_string())?;

        let response_text = response.text().map_err(|err| err.to_string())?;
        Self::parse_embedding(&response_text, &self.response_json_path)
    }
}

#[cfg(test)]
mod tests {
    use super::APIParam;
    use serde_json::json;

    #[test]
    fn request_body_is_json_and_escapes_injected_text() {
        let request_json = json!({
            "input": "<text>",
            "image": "<image_base64>",
            "options": { "dimensions": 3 }
        })
        .as_object()
        .unwrap()
        .clone();
        let param = APIParam {
            target_content_type_prefix: "text/".to_owned(),
            request_endpoint: "https://example.com".to_owned(),
            api_key: None,
            request_json,
            response_json_path: "$.embedding".to_owned(),
        };

        let body = param
            .request_body(Some("hello \"world\"\n"), Some("aGVsbG8="))
            .unwrap();

        assert_eq!(
            body,
            json!({
                "input": "hello \"world\"\n",
                "image": "aGVsbG8=",
                "options": { "dimensions": 3 }
            })
        );
    }

    #[test]
    fn response_json_path_extracts_embedding_from_json_response() {
        let embedding = APIParam::parse_embedding(
            r#"{"data":{"embedding":[0.1,0.2,0.3]}}"#,
            "$.data.embedding",
        )
        .unwrap();

        assert_eq!(embedding, vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn response_json_path_extracts_embedding_slice_from_json_response() {
        let embedding = APIParam::parse_embedding(
            r#"{"data":[{"embedding":[0.1,0.2,0.3,0.4]}]}"#,
            "$.data[0].embedding[:3]",
        )
        .unwrap();

        assert_eq!(embedding, vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn invalid_response_json_returns_an_error() {
        let error = APIParam::parse_embedding(r#"{"embedding":[1,2]"#, "$.embedding").unwrap_err();

        assert!(error.starts_with("response is not valid JSON:"));
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[schemars(title = "JSON Web API Params")]
pub struct Param {
    #[schemars(title = "API List", description = "List of embedding API")]
    pub api_list: Vec<APIParam>,
}

impl Param {
    pub fn get_api_param(&self, mime: &Mime) -> Option<&APIParam> {
        for p in &self.api_list {
            if p.is_processable(mime) {
                return Some(p);
            }
        }
        None
    }
}
