use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[schemars(title = "Xberg Params")]
pub struct Param {
    #[schemars(title = "Text Window", description = "Text length of a single chunk")]
    pub text_window: usize,
    #[schemars(
        title = "Text Overwrap",
        description = "Text overwrap length of chunks"
    )]
    pub text_overwrap: usize,

    #[schemars(
        title = "OCR Languages",
        description = "Languages of OCR, see https://tesseract-ocr.github.io/tessdoc/Data-Files-in-different-versions.html"
    )]
    pub ocr_languages: Vec<String>,
}
