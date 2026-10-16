use std::{
    collections::BTreeMap,
    env::{
        self,
        consts::{ARCH, OS},
    },
    fs,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use tauri_utils::{
    config::{Config, parse},
    platform::Target,
};

fn project_root() -> String {
    let ret = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_string_lossy()
        .to_string();
    ret
}

fn get_config() -> Result<Config, String> {
    let conf_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("invalid path")?
        .parent()
        .ok_or("invalid path")?
        .parent()
        .ok_or("invalid path")?
        .join("src-tauri")
        .join("tauri.conf.json");

    let (parsed, _) = parse(Target::current(), &conf_dir).map_err(|err| err.to_string())?;
    return Ok(parsed);
}

fn app_data_dir(conf: &Config) -> PathBuf {
    dirs::data_dir().unwrap().join(&conf.identifier)
}

fn build_cdylibs(extra: &[String]) -> Result<BTreeMap<String, PathBuf>, io::Error> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut child = Command::new(cargo)
        .current_dir(project_root())
        .args([
            "build",
            "--message-format=json-render-diagnostics",
            "--workspace",
            "--exclude",
            "xtask",
        ])
        .args(extra)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let reader = BufReader::new(child.stdout.take().unwrap());
    let mut libs = BTreeMap::new();

    for line in reader.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&line?) else {
            continue;
        };
        if v["reason"] != "compiler-artifact" {
            continue;
        }
        let is_cdylib = v["target"]["crate_types"]
            .as_array()
            .is_some_and(|k| k.iter().any(|x| x == "cdylib"));
        if !is_cdylib {
            continue;
        }

        let Some(files) = v["filenames"].as_array() else {
            continue;
        };
        let shared = files
            .iter()
            .filter_map(|f| f.as_str())
            .find(|f| is_shared_lib(f));

        if let (Some(name), Some(path)) = (v["target"]["name"].as_str(), shared) {
            libs.insert(name.to_string(), PathBuf::from(path));
        }
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("cargo build failed: {status}"),
        ));
    }
    Ok(libs)
}

fn is_shared_lib(path: &str) -> bool {
    path.ends_with(".so") || path.ends_with(".dylib") || path.ends_with(".dll")
}

fn get_plugin_path(conf: &Config, original_path: &PathBuf) -> Option<PathBuf> {
    let file_stem = original_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let file_ext = original_path
        .extension()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let plugin_types = vec!["embedder", "file_extractor", "file_source", "vector_store"];
    let mut file_name: Option<String> = None;
    let mut dir_name: Option<String> = None;
    for &kw in &plugin_types {
        if file_stem.contains(kw) {
            file_name = Some(file_stem.replace(&format!("lib{kw}_"), "").into());
            dir_name = Some(kw.replace("_", "-"));
            break;
        }
    }
    if let Some(file_name_some) = file_name {
        if let Some(dir_name_some) = dir_name {
            let dir = app_data_dir(conf).join("plugins").join(dir_name_some);
            let _result = fs::create_dir_all(&dir);
            return Some(dir.join(format!("{file_name_some}-{OS}-{ARCH}.{file_ext}")));
        }
    }
    None
}

fn main() {
    let mut args = env::args().skip(1);
    let task = args.next();
    let rest: Vec<String> = args.collect();

    let _result = match task.as_deref() {
        Some("build") => {
            let config = get_config().unwrap();
            let result = build_cdylibs(&rest).unwrap();
            for (_, src_path) in result {
                let dst_path = get_plugin_path(&config, &src_path);
                if let Some(dst_dir_some) = dst_path {
                    fs::copy(src_path, dst_dir_some).unwrap();
                }
            }
        }
        _ => {
            eprintln!("usage: cargo xtask build [cargo build の引数...]");
            std::process::exit(1);
        }
    };
}
