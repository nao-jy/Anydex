use std::{path::Path, sync::Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

/// 非表示・最小化・裏に隠れている、のどれからでも手前に出す
fn bring_main_to_front(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[derive(Clone, Serialize)]
struct Launch {
    args: Vec<String>, // 実行ファイルを除いた引数。存在するパスは絶対パスに直す
    cwd: String,       // その起動の作業フォルダー
}

/// 画面が受け取るまで、起動の引数をためておく
#[derive(Default)]
struct PendingLaunches(Mutex<Vec<Launch>>);

fn queue_launch(app: &AppHandle, argv: Vec<String>, cwd: String) {
    let args = argv
        .into_iter()
        .skip(1) // 先頭は実行ファイル自身のパス
        .map(|arg| {
            let path = Path::new(&cwd).join(&arg); // arg が絶対パスなら arg のまま
            if path.exists() {
                path.to_string_lossy().into_owned()
            } else {
                arg
            }
        })
        .collect();
    app.state::<PendingLaunches>()
        .0
        .lock()
        .unwrap()
        .push(Launch { args, cwd });
}

/// 2 回目の起動のたびに、1 回目のプロセスで呼ばれる
pub fn on_second_instance(app: &AppHandle, argv: Vec<String>, cwd: String) {
    queue_launch(app, argv, cwd);
    let _ = app.emit("launch-queued", ()); // 中身は take_launches で取りに来てもらう
    bring_main_to_front(app);
}
