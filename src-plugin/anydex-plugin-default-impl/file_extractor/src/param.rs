use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Param {
    #[schemars(description = "text length of a single chunk")]
    pub text_window: i32,
    #[schemars(description = "text overwrap length of chunks")]
    pub text_overwrap: i32,
}

impl Param {
    pub fn split_text(&self, text: &str) -> Result<Vec<String>, String> {
        let window_length = self.text_window;
        let overwrap_length = self.text_overwrap;
        if window_length <= 0 {
            return Err(format!("text_window {window_length} is invalid"));
        }
        if overwrap_length >= window_length {
            return Err(format!("text_window is less than text_overwrap"));
        }

        let chars: Vec<char> = text.chars().collect();
        let step = (window_length - overwrap_length) as usize;

        Ok(chars
            .windows(window_length as usize)
            .step_by(step)
            .map(|window| window.iter().collect())
            .collect())
    }
}
