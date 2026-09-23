use jsonpath_rust::JsonPath;
use mime_guess::Mime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct APIParam {
    #[schemars(description = "content type prefix to prcess with this API")]
    pub target_content_type_prefix: String,

    #[schemars(description = "endpoint for request")]
    pub request_endpoint: String,

    #[schemars(
        description = "JSON format for request. {{text}} {{image_base64}} are reserved and automatically injected"
    )]
    pub request_json: serde_json::Value,

    #[schemars(description = "place of embedding in response")]
    pub response_json_path: String,
}

impl APIParam {
    pub fn is_processable(&self, mime: &Mime) -> bool {
        mime.to_string()
            .to_lowercase()
            .starts_with(&self.target_content_type_prefix)
    }
    pub fn get_text_replaced(req_body: &str, text: &str) -> String {
        req_body.replace("{{text}}", text)
    }
    pub fn get_image_base64_replaced(req_body: &str, image_base64: &str) -> String {
        req_body.replace("{{image_base64}}", image_base64)
    }
    pub fn try_get_embedding(
        &self,
        text: Option<String>,
        image_base64: Option<String>,
    ) -> Result<Vec<f32>, String> {
        let mut body_json = self.request_json.to_string();
        if let Some(content) = text {
            body_json = APIParam::get_text_replaced(&body_json, &content);
        }
        if let Some(content) = image_base64 {
            body_json = APIParam::get_image_base64_replaced(&body_json, &content);
        }

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&self.request_endpoint)
            .body(body_json)
            .send()
            .map_err(|err| err.to_string())?;

        let response_text = response.text().map_err(|err| err.to_string())?;
        let response_json = serde_json::Value::from(response_text);

        let result = response_json
            .query(&self.response_json_path)
            .map_err(|err| err.to_string())?;
        let embedding = result.get(0);
        if let Some(result_array) = embedding {
            let ret = match result_array {
                serde_json::Value::Array(arr) => {
                    let fvec: Vec<f32> = arr
                        .iter()
                        .map(|v| v.as_f64().unwrap_or_default() as f32)
                        .collect();
                    Ok(fvec)
                }
                _ => Err(format!("response is not as expected: {response_json}")),
            };
            return ret;
        }
        Err(format!("response is not as expected: {response_json}"))
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Param {
    #[schemars(description = "list of embedding API")]
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
