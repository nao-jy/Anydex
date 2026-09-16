use std::{collections::HashMap, fs, path::Path};

pub trait Manager<T> {
    fn init(&mut self, plugin_dir: &Path) -> Result<(), String>;
    fn get_plugin_names(&self) -> Vec<String>;
    fn get_plugin(&mut self, name: &str) -> Option<&mut T>;
}

pub struct Plugin<T> {
    // Fields are dropped in declaration order, so the instance is dropped
    // before the library it may call into from its destructor.
    pub instance: Box<T>,
    pub library: libloading::Library,
}

pub struct PluginManager<T> {
    plugins: HashMap<String, Plugin<T>>,
}

pub trait LibraryLoader: Sized {
    fn load_library(library: &libloading::Library) -> Result<Self, String>;
}

impl<T: LibraryLoader> Manager<T> for PluginManager<T> {
    fn init(&mut self, plugin_dir: &Path) -> Result<(), String> {
        let entries = fs::read_dir(plugin_dir).map_err(|error| {
            format!(
                "failed to read plugin directory '{}': {error}",
                plugin_dir.display()
            )
        })?;
        let mut plugins = HashMap::new();

        for entry in entries {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to read an entry in plugin directory '{}': {error}",
                    plugin_dir.display()
                )
            })?;
            let path = entry.path();

            if !path.is_file() || !is_dynamic_library(&path) {
                continue;
            }

            let name = entry.file_name().to_string_lossy().into_owned();
            // Loading a dynamic library is unsafe because its initialization code runs in-process.
            let library = unsafe { libloading::Library::new(&path) }
                .map_err(|error| format!("failed to load plugin '{}': {error}", path.display()))?;
            let instance = T::load_library(&library).map_err(|error| {
                format!("failed to initialize plugin '{}': {error}", path.display())
            })?;

            plugins.insert(
                name,
                Plugin {
                    instance: Box::new(instance),
                    library,
                },
            );
        }

        self.plugins = plugins;
        Ok(())
    }

    fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    fn get_plugin(&mut self, name: &str) -> Option<&mut T> {
        self.plugins
            .get_mut(name)
            .map(|plugin| plugin.instance.as_mut())
    }
}

fn is_dynamic_library(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    file_name.ends_with(std::env::consts::DLL_SUFFIX)
}
