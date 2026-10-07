use std::sync::{Arc, Mutex};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use oauth2::CsrfToken;
use serde::Deserialize;
use subtle::ConstantTimeEq;
use tokio::sync::oneshot::{self, Receiver};

type CallbackResult = std::result::Result<String, String>;

#[derive(Deserialize)]
struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

struct AppState {
    expected_state: String,
    tx: Mutex<Option<oneshot::Sender<CallbackResult>>>,
}

pub const CALLBACK_PATH: &str = "/callback";

async fn callback(
    State(st): State<Arc<AppState>>,
    Query(p): Query<CallbackParams>,
) -> (StatusCode, Html<String>) {
    // state が一致しないリクエスト(CSRF・他プロセスからの偽リクエスト等)は無視。
    // フロー自体は中断せず、正規のコールバックを待ち続ける。
    let state_ok = p
        .state
        .as_deref()
        .map(|s| bool::from(s.as_bytes().ct_eq(st.expected_state.as_bytes())))
        .unwrap_or(false);
    if !state_ok {
        return (
            StatusCode::BAD_REQUEST,
            page("Invalid request", "state が一致しません。"),
        );
    }

    let result: CallbackResult = match (p.error, p.code) {
        (Some(err), _) => Err(match p.error_description {
            Some(d) => format!("{err}: {d}"),
            None => err,
        }),
        (None, Some(code)) => Ok(code),
        (None, None) => Err("認可コードがありません".to_string()),
    };

    let response = match &result {
        Ok(_) => (
            StatusCode::OK,
            page("ログイン完了", "タブを閉じてください。"),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, page("ログイン失敗", e)),
    };

    let mut mtx = st.tx.lock().unwrap();
    if let Some(v) = mtx.take() {
        let _ = v.send(result);
    }
    response
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn page(title: &str, body: &str) -> Html<String> {
    Html(format!(
        "<!doctype html><html lang=\"ja\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>{t}</title>\
         <style>body{{font-family:system-ui,sans-serif;max-width:32rem;margin:20vh auto;padding:0 1rem;text-align:center}}</style>\
         </head><body><h1>{t}</h1><p>{b}</p></body></html>",
        t = html_escape(title),
        b = html_escape(body),
    ))
}

pub fn get_server<T>(csrf_token: &CsrfToken) -> (Router<T>, Receiver<Result<String, String>>) {
    let (code_tx, code_rx) = oneshot::channel::<CallbackResult>();
    let app_state = Arc::new(AppState {
        expected_state: csrf_token.secret().to_owned(),
        tx: Mutex::new(Some(code_tx)),
    });

    let app = Router::new()
        .route(CALLBACK_PATH, get(callback))
        .with_state(app_state);
    return (app, code_rx);
}
