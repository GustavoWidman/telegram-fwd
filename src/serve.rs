use actix_web::{Error, HttpResponse, get, web};
use serde::Deserialize;
use serde_json::json;

use crate::AppState;

#[derive(Deserialize)]
struct Query {
    file: usize,
}

#[get("/download/{file}")]
async fn download(
    query: web::Path<Query>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let files = data.files.clone();
    let file: Option<crate::file::DownloadFile> = files.get(query.file).cloned();

    return Ok(match file {
        Some(file) => {
            let name = file.name.clone();
            let size = file.file_size.to_string();
            let stream = file.download_stream(data.client.clone());

            HttpResponse::Ok()
                .append_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", name),
                ))
                .append_header(("Content-Length", size))
                .content_type("application/octet-stream")
                .streaming(stream)
        }
        None => HttpResponse::NotFound().json(json! {{
            "error": "File not found"
        }}),
    });
}

#[get("/")]
async fn index(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let index = include_str!("../templates/index.html");

    let files = data
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            format!(
                "<a href=\"/download/{}\">{}</a><br>",
                i,
                format!(
                    "{} ({})",
                    file.name,
                    crate::utils::bytes_to_pretty_string(file.file_size)
                )
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    Ok(HttpResponse::Ok().body(index.replace("{{ files }}", &files)))
}
