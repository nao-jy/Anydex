use anydex_plugin_api_lib::{
    embedder::interface::DynamicEmbedder, file_extractor::interface::DynamicFileExtractor,
    file_source::interface::DynamicFileSource, vector_store::interface::DynamicVectorStore,
};
use enum_iterator::{all, Sequence};
use std::{path::PathBuf, sync::Mutex};

use crate::plugin::manager::PluginManager;

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
        {
            let mut base_dir_mtx = BASE_DIR.lock().unwrap();
            *base_dir_mtx = Some(String::from(base_dir));
        }

        // create dir
        for plugin_type in all::<Self>() {
            std::fs::create_dir_all(&plugin_type.get_dir()).map_err(|err| err.to_string())?;
        }

        // init plugin manager
        {
            let mut mgr_init =
                PluginManager::<DynamicFileSource>::init(&Self::FileSource.get_dir());
            mgr_init.load_plugins()?;
            let mut mgr = FILE_SOURCE_MGR.lock().unwrap();
            *mgr = Some(mgr_init)
        }
        {
            let mut mgr_init =
                PluginManager::<DynamicFileExtractor>::init(&Self::FileExtractor.get_dir());
            mgr_init.load_plugins()?;
            let mut mgr = FILE_EXTRACTOR_MGR.lock().unwrap();
            *mgr = Some(mgr_init)
        }
        {
            let mut mgr_init = PluginManager::<DynamicEmbedder>::init(&Self::Embedder.get_dir());
            mgr_init.load_plugins()?;
            let mut mgr = EMBEDDER_MGR.lock().unwrap();
            *mgr = Some(mgr_init)
        }
        {
            let mut mgr_init =
                PluginManager::<DynamicVectorStore>::init(&Self::VectorStore.get_dir());
            mgr_init.load_plugins()?;
            let mut mgr = VECTOR_STORE_MGR.lock().unwrap();
            *mgr = Some(mgr_init)
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
        match self {
            Self::Embedder => {
                let mgr_mtx = EMBEDDER_MGR.lock().unwrap();
                let mgr = mgr_mtx.as_ref().unwrap();
                return mgr.get_plugin_names();
            }
            Self::FileExtractor => {
                let mgr_mtx = FILE_EXTRACTOR_MGR.lock().unwrap();
                let mgr = mgr_mtx.as_ref().unwrap();
                return mgr.get_plugin_names();
            }
            Self::FileSource => {
                let mgr_mtx = FILE_SOURCE_MGR.lock().unwrap();
                let mgr = mgr_mtx.as_ref().unwrap();
                return mgr.get_plugin_names();
            }
            Self::VectorStore => {
                let mgr_mtx = VECTOR_STORE_MGR.lock().unwrap();
                let mgr = mgr_mtx.as_ref().unwrap();
                return mgr.get_plugin_names();
            }
        }
    }

    pub fn reload(self) -> Result<(), String> {
        match self {
            Self::Embedder => {
                let mut mgr_mtx = EMBEDDER_MGR.lock().unwrap();
                let mgr = mgr_mtx.as_mut().unwrap();
                mgr.load_plugins()?;
                Ok(())
            }
            Self::FileExtractor => {
                let mut mgr_mtx = FILE_EXTRACTOR_MGR.lock().unwrap();
                let mgr = mgr_mtx.as_mut().unwrap();
                mgr.load_plugins()?;
                Ok(())
            }
            Self::FileSource => {
                let mut mgr_mtx = FILE_SOURCE_MGR.lock().unwrap();
                let mgr = mgr_mtx.as_mut().unwrap();
                mgr.load_plugins()?;
                Ok(())
            }
            Self::VectorStore => {
                let mut mgr_mtx = VECTOR_STORE_MGR.lock().unwrap();
                let mgr = mgr_mtx.as_mut().unwrap();
                mgr.load_plugins()?;
                Ok(())
            }
        }
    }
}
