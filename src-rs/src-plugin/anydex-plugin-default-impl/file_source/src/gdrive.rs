use std::path::Path;

use google_drive3::{hyper_rustls, hyper_util, DriveHub};
use http_body_util::BodyExt;

pub fn native_export_format(content_type: &str) -> Option<(&'static str, &'static str)> {
    match content_type {
        "application/vnd.google-apps.document" => Some((
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "docx",
        )),
        "application/vnd.google-apps.spreadsheet" => Some((
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "xlsx",
        )),
        "application/vnd.google-apps.presentation" => Some((
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "pptx",
        )),
        _ => None,
    }
}

fn drive_hub(
    access_token: &str,
) -> Result<DriveHub<impl google_drive3::common::Connector>, String> {
    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .map_err(|err| format!("failed to load native TLS roots: {err}"))?
        .https_only()
        .enable_http2()
        .build();
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(connector);

    Ok(DriveHub::new(client, access_token.to_owned()))
}

async fn find_gdrive_file_id_with_hub<C: google_drive3::common::Connector>(
    hub: &DriveHub<C>,
    root: &str,
    path: &str,
) -> Result<String, String> {
    let segments: Vec<_> = root
        .split('/')
        .chain(path.split('/'))
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return Err("cannot export a Google Drive file with an empty path".into());
    }

    let mut parent_id = "root".to_owned();
    for (index, segment) in segments.iter().enumerate() {
        let escaped_name = segment.replace('\\', "\\\\").replace('\'', "\\'");
        let query =
            format!("name = '{escaped_name}' and '{parent_id}' in parents and trashed = false");
        let (_, files) = hub
            .files()
            .list()
            .q(&query)
            .page_size(1000)
            .order_by("modifiedTime desc,createdTime desc")
            .param("fields", "files(id,mimeType)")
            .doit()
            .await
            .map_err(|err| format!("Google Drive file lookup failed: {err}"))?;
        let file = files
            .files
            .unwrap_or_default()
            .into_iter()
            .next()
            .ok_or_else(|| format!("Google Drive path segment '{segment}' was not found"))?;

        if index + 1 < segments.len() {
            if file.mime_type.as_deref() != Some("application/vnd.google-apps.folder") {
                return Err(format!(
                    "Google Drive path segment '{segment}' is not a folder"
                ));
            }
            parent_id = file
                .id
                .ok_or_else(|| format!("Google Drive folder '{segment}' has no ID"))?;
        } else {
            return file
                .id
                .ok_or_else(|| format!("Google Drive file '{segment}' has no ID"));
        }
    }

    Err("Google Drive file path could not be resolved".into())
}

pub async fn find_gdrive_file_id(
    access_token: &str,
    root: &str,
    path: &str,
) -> Result<String, String> {
    let hub = drive_hub(access_token)?;
    find_gdrive_file_id_with_hub(&hub, root, path).await
}

pub async fn export_gdrive_file(
    access_token: &str,
    root: &str,
    path: &str,
    content_type: &str,
) -> Result<(Vec<u8>, String, String), String> {
    let (export_mime_type, extension) = native_export_format(content_type)
        .ok_or_else(|| format!("unsupported Google Drive native file type: {content_type}"))?;
    let hub = drive_hub(access_token)?;
    let file_id = find_gdrive_file_id_with_hub(&hub, root, path).await?;
    let response = hub
        .files()
        .export(&file_id, export_mime_type)
        .doit()
        .await
        .map_err(|err| format!("Google Drive export request failed: {err}"))?;
    let content = response
        .into_body()
        .collect()
        .await
        .map_err(|err| format!("failed to read Google Drive export: {err}"))?
        .to_bytes()
        .to_vec();
    let filename = Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path);
    let exported_name = Path::new(filename)
        .with_extension(extension)
        .to_string_lossy()
        .into_owned();

    Ok((content, export_mime_type.to_owned(), exported_name))
}
