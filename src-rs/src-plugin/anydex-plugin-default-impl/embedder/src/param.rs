use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum BackendDevice {
    Cpu,
    Cuda,
    Metal,
}

fn default_backend_device() -> BackendDevice {
    BackendDevice::Cpu
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[schemars(title = "Qwen3-VL Params")]
pub struct Param {
    #[schemars(title = "Vector Dimensions", range(min = 64, max = 2048))]
    pub dimensions: i32,

    #[serde(default = "default_backend_device")]
    #[schemars(title = "Backend Device", default = "default_backend_device")]
    pub device: BackendDevice,
}

#[cfg(test)]
mod tests {
    use super::Param;

    #[test]
    fn defaults_max_length_for_existing_settings() {
        let params: Param = serde_json::from_str(r#"{"dimensions":768,"device":"Metal"}"#).unwrap();

        assert_eq!(params.dimensions, 768);
    }
}
