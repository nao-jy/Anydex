use std::{collections::HashMap, fs, path::Path};

use anydex_plugin_api_lib::common::{get_dylib_stem_name, get_dylib_suffix};
use tracing::warn;

pub struct Plugin<T> {
    // Fields are dropped in declaration order, so the instance is dropped
    // before the library it may call into from its destructor.
    pub instance: Box<T>,
    pub library: libloading::Library,
}

pub struct PluginManager<T> {
    plugins: HashMap<String, Plugin<T>>,
    plugin_dir: String,
}

pub trait LibraryLoader: Sized {
    fn load_library(library: &libloading::Library) -> Result<Self, String>;
}

impl<T: LibraryLoader> PluginManager<T> {
    pub fn init(plugin_dir: &str) -> Self {
        let ret = PluginManager::<T> {
            plugins: HashMap::<String, Plugin<T>>::new(),
            plugin_dir: String::from(plugin_dir),
        };
        return ret;
    }
    pub fn load_plugins(&mut self) -> Result<(), String> {
        let entries = fs::read_dir(&self.plugin_dir).map_err(|error| {
            format!(
                "failed to read plugin directory '{}': {error}",
                self.plugin_dir
            )
        })?;
        let mut plugins = HashMap::new();

        for entry in entries {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to read an entry in plugin directory '{}': {error}",
                    self.plugin_dir
                )
            })?;
            let path = entry.path();

            if !path.is_file() || !is_valid_name(&path) {
                continue;
            }

            let name_opt = get_dylib_stem_name(entry.file_name().to_str().unwrap_or_default());
            if let Some(name) = name_opt {
                let library_load = unsafe { libloading::Library::new(&path) };
                if let Err(err) = library_load {
                    let pathname = path.to_str().unwrap_or("?");
                    warn!("failed to load plugin {pathname} {err}");
                    continue;
                }
                let library = library_load.unwrap();

                let load_lib = T::load_library(&library);
                if let Err(err) = load_lib {
                    let pathname = path.to_str().unwrap_or("?");
                    warn!("failed to load plugin {pathname} {err}");
                    continue;
                }
                let instance = load_lib.unwrap();

                plugins.insert(
                    name,
                    Plugin {
                        instance: Box::new(instance),
                        library,
                    },
                );
            }
        }

        self.plugins = plugins;
        Ok(())
    }

    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    pub fn get_plugin(&mut self, name: &str) -> Option<&mut T> {
        self.plugins
            .get_mut(name)
            .map(|plugin| plugin.instance.as_mut())
    }
}

fn is_valid_name(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    file_name.ends_with(&get_dylib_suffix())
}
