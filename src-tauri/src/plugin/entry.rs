use enum_iterator::{all, Sequence};
use std::{path::PathBuf, sync::Mutex};

use crate::plugin::{
    embedder::implement::DynamicEmbedder, file_extractor::implement::DynamicFileExtractor,
    file_source::implement::DynamicFileSource, manager::PluginManager,
    vector_store::implement::DynamicVectorStore,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Sequence)]
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

impl TryFrom<&str> for PluginType {
    fn try_from(plugin_type: &str) -> Result<PluginType, String> {
        match plugin_type {
            "file-source" => Ok(PluginType::FileSource),
            "file-extractor" => Ok(PluginType::FileExtractor),
            "embedder" => Ok(PluginType::Embedder),
            "vector-store" => Ok(PluginType::VectorStore),
            _ => Err(format!("unknown plugin type: {plugin_type}")),
        }
    }

    type Error = String;
}

static BASE_DIR: Mutex<Option<String>> = Mutex::new(None);
pub static FILE_SOURCE_MGR: Mutex<Option<PluginManager<DynamicFileSource>>> = Mutex::new(None);
pub static FILE_EXTRACTOR_MGR: Mutex<Option<PluginManager<DynamicFileExtractor>>> =
    Mutex::new(None);
pub static EMBEDDER_MGR: Mutex<Option<PluginManager<DynamicEmbedder>>> = Mutex::new(None);
pub static VECTOR_STORE_MGR: Mutex<Option<PluginManager<DynamicVectorStore>>> = Mutex::new(None);

impl PluginType {
    pub fn init(base_dir: &str) -> Result<(), String> {
        // init base_dir
        let mut base_dir_mtx = BASE_DIR.lock().unwrap();
        *base_dir_mtx = Some(String::from(base_dir));

        // init plugin manager
        {
            let mut mgr = FILE_SOURCE_MGR.lock().unwrap();
            let mut mgr_init =
                PluginManager::<DynamicFileSource>::init(&Self::FileSource.get_dir());
            mgr_init.load_plugins()?;
            *mgr = Some(mgr_init)
        }
        {
            let mut mgr = FILE_EXTRACTOR_MGR.lock().unwrap();
            let mut mgr_init =
                PluginManager::<DynamicFileExtractor>::init(&Self::FileExtractor.get_dir());
            mgr_init.load_plugins()?;
            *mgr = Some(mgr_init)
        }
        {
            let mut mgr: std::sync::MutexGuard<'_, Option<PluginManager<DynamicEmbedder>>> =
                EMBEDDER_MGR.lock().unwrap();
            let mut mgr_init = PluginManager::<DynamicEmbedder>::init(&Self::Embedder.get_dir());
            mgr_init.load_plugins()?;
            *mgr = Some(mgr_init)
        }
        {
            let mut mgr = VECTOR_STORE_MGR.lock().unwrap();
            let mut mgr_init =
                PluginManager::<DynamicVectorStore>::init(&Self::VectorStore.get_dir());
            mgr_init.load_plugins()?;
            *mgr = Some(mgr_init)
        }

        // create dir
        for plugin_type in all::<Self>() {
            std::fs::create_dir_all(&plugin_type.get_dir());
        }

        Ok(())
    }

    pub fn get_dir(self) -> String {
        let base_dir_opt_mtx = BASE_DIR.lock().unwrap();
        let plugin_name: String = self.into();
        let base_dir_opt = base_dir_opt_mtx.clone();
        PathBuf::from(base_dir_opt.unwrap())
            .join("plugins")
            .join(plugin_name)
            .display()
            .to_string()
    }

    pub fn list_names(self) -> Vec<String> {
        vec!["sample".into()]
    }
}
