use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Param {
    #[schemars(description = "service name from https://opendal.apache.org/services/")]
    pub service_name: String,
    #[schemars(description = "service configurations from https://opendal.apache.org/services/")]
    pub service_configurations: serde_json::Value,

    // following definitions will be used to automatically map OAuth credentials
    #[schemars(description = "oauth access_token name in service_configurations")]
    pub oauth_conf_access_token_name: Option<String>,
    #[schemars(description = "oauth refresh_token name in service_configurations")]
    pub oauth_conf_refresh_token_name: Option<String>,
    #[schemars(description = "oauth cliend_id name in service_configurations")]
    pub oauth_conf_cliend_id_name: Option<String>,
}
