use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[schemars(title = "OpenDAL Params")]
pub struct Param {
    #[schemars(
        title = "Service Name",
        description = "Service name from https://opendal.apache.org/services/"
    )]
    pub service_name: String,
    #[schemars(
        title = "Service Configurations",
        description = "Service configurations from https://opendal.apache.org/services/"
    )]
    pub service_configurations: serde_json::Map<String, serde_json::Value>,

    // following definitions will be used to automatically map OAuth credentials
    #[schemars(
        title = "OAuth access_token Name",
        description = "OAuth access_token name in service_configurations"
    )]
    pub oauth_conf_access_token_name: Option<String>,
    #[schemars(
        title = "OAuth refresh_token Name",
        description = "OAuth refresh_token name in service_configurations"
    )]
    pub oauth_conf_refresh_token_name: Option<String>,
    #[schemars(
        title = "OAuth cliend_id Name",
        description = "OAuth cliend_id name in service_configurations"
    )]
    pub oauth_conf_cliend_id_name: Option<String>,
}
