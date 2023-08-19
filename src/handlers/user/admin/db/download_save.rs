use std::sync::Arc;

use crate::{
    utils::{
        check_auth_token::check_auth_token, get_token_data::get_token_data,
        has_permission::has_permission,
    },
    AppState,
};
use axum::{
    body::StreamBody,
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use mongodb::{
    bson::{doc, Document},
    Collection,
};

use tokio_util::io::ReaderStream;

use std::io::prelude::*;
use std::io::{Seek, Write};
use std::iter::Iterator;
use zip::write::FileOptions;

use std::fs::File;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadSaveQuery {
    save_id: String,
    token: String,
}

pub async fn download_save_handler(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<DownloadSaveQuery>,
) -> impl IntoResponse {
    let save_id = query.save_id;
    let token = query.token;
    let valid = check_auth_token(app_state.clone(), token.clone().to_string());
    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid token or token expired"));
    }

    let token_data = get_token_data(app_state.clone(), token.to_string());

    let has_perm = has_permission(
        token_data.user_id,
        "administrator".to_string(),
        app_state.clone(),
    )
    .await;

    if !has_perm {
        return Err((
            StatusCode::FORBIDDEN,
            "You don't have administrator permission",
        ));
    }

    let db = app_state.db.clone();
    let collection: Collection<Document> = db.collection("db_saves");

    let document = collection
        .find_one(
            doc! {"_id": mongodb::bson::oid::ObjectId::parse_str(&save_id).unwrap()},
            None,
        )
        .await
        .unwrap()
        .unwrap();

    let time = document.get("time").unwrap().as_i64().unwrap();
    let db_id = document.get("db_id").unwrap().as_object_id().unwrap();

    let path = format!("db_saves/{}/{}", db_id.clone(), time.clone());

    // Create zip
    let file = match File::create("save.zip") {
        Ok(file) => file,
        Err(_err) => return Err((StatusCode::NOT_FOUND, "File not found")),
    };
    let it = WalkDir::new(path.clone()).into_iter();

    zip_dir(
        &mut it.filter_map(|e| e.ok()),
        &path,
        file,
        zip::CompressionMethod::Stored,
    )
    .unwrap();

    let file = match tokio::fs::File::open("save.zip").await {
        Ok(file) => file,
        Err(_err) => return Err((StatusCode::NOT_FOUND, "File not found")),
    };

    // Convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // Convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/zip"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"save.zip\"",
        ),
    ];

    Ok((headers, body))
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}
