use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PluginType {
    FileSource,
    FileExtractor,
    Embedder,
    VectorStore,
}

impl Into<String> for PluginType {
    fn into(self) -> String {
        match self {
            PluginType::FileSource => "file-source".into(),
            PluginType::FileExtractor => "file-extractor".into(),
            PluginType::Embedder => "embedder".into(),
            PluginType::VectorStore => "vector-store".into(),
        }
    }
}

impl PluginType {
    pub fn get_dir(self, base_dir: &PathBuf) -> String {
        let plugin_name: String = self.into();
        base_dir
            .join("plugins")
            .join(plugin_name)
            .display()
            .to_string()
    }

    pub fn list_names(self, base_dir: &PathBuf) -> Vec<String> {
        vec!["sample".into()]
    }
}
