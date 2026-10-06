use std::{path::Path, sync::LazyLock};

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

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

#[derive(serde::Deserialize)]
struct GdriveFileId {
    id: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
}

#[derive(serde::Deserialize)]
struct GdriveFileIdList {
    files: Vec<GdriveFileId>,
}

pub async fn find_gdrive_file_id(
    access_token: &str,
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
        let response = HTTP_CLIENT
            .get("https://www.googleapis.com/drive/v3/files")
            .bearer_auth(access_token)
            .query(&[
                ("q", query.as_str()),
                ("pageSize", "1000"),
                ("orderBy", "modifiedTime desc,createdTime desc"),
                ("fields", "files(id,mimeType)"),
            ])
            .send()
            .await
            .map_err(|err| format!("Google Drive file lookup failed: {err}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Google Drive file lookup returned {status}: {body}"
            ));
        }
        let files: GdriveFileIdList = response
            .json()
            .await
            .map_err(|err| format!("invalid Google Drive file lookup response: {err}"))?;
        let file = files
            .files
            .into_iter()
            .next()
            .ok_or_else(|| format!("Google Drive path segment '{segment}' was not found"))?;

        if index + 1 < segments.len() {
            if file.mime_type != "application/vnd.google-apps.folder" {
                return Err(format!(
                    "Google Drive path segment '{segment}' is not a folder"
                ));
            }
            parent_id = file.id;
        } else {
            return Ok(file.id);
        }
    }

    Err("Google Drive file path could not be resolved".into())
}

pub async fn export_gdrive_file(
    access_token: &str,
    root: &str,
    path: &str,
    content_type: &str,
) -> Result<(Vec<u8>, String, String), String> {
    let (export_mime_type, extension) = native_export_format(content_type)
        .ok_or_else(|| format!("unsupported Google Drive native file type: {content_type}"))?;
    let file_id = find_gdrive_file_id(access_token, root, path).await?;
    let url = format!("https://www.googleapis.com/drive/v3/files/{file_id}/export");
    let response = HTTP_CLIENT
        .get(url)
        .bearer_auth(access_token)
        .query(&[("mimeType", export_mime_type)])
        .send()
        .await
        .map_err(|err| format!("Google Drive export request failed: {err}"))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Google Drive export returned {status}: {body}"));
    }
    let content = response
        .bytes()
        .await
        .map_err(|err| format!("failed to read Google Drive export: {err}"))?
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
