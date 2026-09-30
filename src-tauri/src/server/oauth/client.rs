use std::time::Duration;

use chrono::{DateTime, Utc};
use oauth2::{
    basic::BasicClient, reqwest, AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, PkceCodeChallenge, RedirectUrl, RefreshToken, RevocationUrl, Scope,
    StandardRevocableToken, TokenResponse, TokenUrl,
};

use crate::server::{
    manager::{end, start},
    oauth::server::get_server,
};

pub struct OAuthResult {
    pub access_token: AccessToken,
    pub refresh_token: Option<RefreshToken>,
    pub expires_at: Option<DateTime<Utc>>,
}

fn get_server_name(client_id: &str) -> String {
    format!("oauth-{client_id}")
}

pub async fn do_oauth_pkce(
    client_id: &str,
    client_secret: Option<&str>,
    auth_endpoint: &str,
    token_endpoint: &str,
    scopes: &[String],
) -> Result<OAuthResult, String> {
    let server_name = get_server_name(client_id);
    let result = do_oauth_pkce_inner(
        client_id,
        client_secret,
        auth_endpoint,
        token_endpoint,
        scopes,
    )
    .await;
    let _ = end(&server_name);
    result
}

async fn do_oauth_pkce_inner(
    client_id: &str,
    client_secret: Option<&str>,
    auth_endpoint: &str,
    token_endpoint: &str,
    scopes: &[String],
) -> Result<OAuthResult, String> {
    let mut client = BasicClient::new(ClientId::new(client_id.to_string()))
        .set_auth_uri(AuthUrl::new(auth_endpoint.to_string()).map_err(|err| err.to_string())?)
        .set_token_uri(TokenUrl::new(token_endpoint.to_string()).map_err(|err| err.to_string())?);

    if let Some(v) = client_secret {
        client = client.set_client_secret(ClientSecret::new(v.to_string()));
    }

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(scopes.iter().cloned().map(Scope::new))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // start server
    let (app, code_receiver) = get_server(&csrf_token);
    let server_name = get_server_name(client_id);
    let _end_result = end(&server_name);
    let start_result = start(&server_name, app, None).map_err(|err| err.to_string())?;

    let redirect_url = start_result.to_string() + "/callback";

    client = client
        .set_redirect_uri(RedirectUrl::new(redirect_url.clone()).map_err(|err| err.to_string())?);

    webbrowser::open(&auth_url.to_string()).unwrap();

    let timeout = Duration::from_secs(300);
    let outcome = tokio::time::timeout(timeout.clone(), code_receiver).await;
    let code = match outcome {
        Err(_) => Err(format!("ログインがタイムアウトしました")),
        Ok(Err(_)) => Err(format!("コールバック待ち受けが予期せず終了しました")),
        Ok(Ok(Err(e))) => Err(format!("認可エラー: {e}")),
        Ok(Ok(Ok(code))) => Ok(code),
    }?;

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .map_err(|err| err.to_string())?;

    let _end_result = end(&server_name);

    Ok(OAuthResult {
        access_token: token_result.access_token().clone(),
        refresh_token: token_result
            .refresh_token()
            .map_or(None, |t| Some(t.clone())),
        expires_at: token_result
            .expires_in()
            .map_or(None, |e| Some(Utc::now() + e)),
    })
}

async fn revoke_inner(
    client_id: &str,
    client_secret: Option<&str>,
    token: &StandardRevocableToken,
    revocation_endtpoint: &str,
) -> Result<(), String> {
    let mut client = BasicClient::new(ClientId::new(client_id.to_string())).set_revocation_url(
        RevocationUrl::new(revocation_endtpoint.to_string()).map_err(|err| err.to_string())?,
    );
    if let Some(v) = client_secret {
        client = client.set_client_secret(ClientSecret::new(v.to_string()));
    }

    let http_client = reqwest::ClientBuilder::new()
        // SSRF対策としてリダイレクトは無効化
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|err| err.to_string())?;

    // アクセストークンをrevoke
    client
        .revoke_token(token.to_owned())
        .map_err(|err| err.to_string())?
        .request_async(&http_client)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn revoke_access_token(
    client_id: &str,
    client_secret: Option<&str>,
    access_token: &str,
    revocation_endtpoint: &str,
) -> Result<(), String> {
    revoke_inner(
        client_id,
        client_secret,
        &StandardRevocableToken::AccessToken(AccessToken::new(access_token.to_string())),
        revocation_endtpoint,
    )
    .await
}

pub async fn revoke_refresh_token(
    client_id: &str,
    client_secret: Option<&str>,
    refresh_token: &str,
    revocation_endtpoint: &str,
) -> Result<(), String> {
    revoke_inner(
        client_id,
        client_secret,
        &StandardRevocableToken::RefreshToken(RefreshToken::new(refresh_token.to_string())),
        revocation_endtpoint,
    )
    .await
}
