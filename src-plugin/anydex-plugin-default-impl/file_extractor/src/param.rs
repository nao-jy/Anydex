use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Param {
    #[schemars(description = "text length of a single chunk")]
    pub text_window: usize,
    #[schemars(description = "text overwrap length of chunks")]
    pub text_overwrap: usize,

    #[schemars(
        description = "languages of OCR, see https://tesseract-ocr.github.io/tessdoc/Data-Files-in-different-versions.html"
    )]
    pub ocr_languages: Vec<String>,
}
