use actix_web::{
    Error, HttpRequest, HttpResponse, get,
    http::header::{self, ContentType, TryIntoHeaderValue},
    post, web,
};
use serde::Deserialize;

use crate::AppState;

#[derive(Deserialize)]
struct Query {
    file: usize,
}

#[derive(Deserialize)]
struct Login {
    password: String,
}

#[get("/download/{file}")]
async fn download(
    request: HttpRequest,
    query: web::Path<Query>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    if !request
        .cookie("password")
        .and_then(|cookie| {
            cookie
                .value()
                .parse::<String>()
                .ok()
                .map(|v| v == data.config.password)
        })
        .map_or(false, |v| v)
    {
        return login_index().await;
    }

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
        None => {
            let error_html = include_str!("../templates/error.html");
            HttpResponse::Ok()
                .append_header((header::CONTENT_TYPE, ContentType::html().try_into_value()?))
                .body(error_html.replace("{{ error }}", "File not found"))
        }
    });
}

#[get("/")]
async fn index(request: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    if request
        .cookie("password")
        .and_then(|cookie| {
            cookie
                .value()
                .parse::<String>()
                .ok()
                .map(|v| v == data.config.password)
        })
        .map_or(false, |v| v)
    {
        return main(data).await;
    } else {
        login_index().await
    }
}

async fn main(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let html = include_str!("../templates/index.html");

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

    Ok(HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, ContentType::html().try_into_value()?))
        .body(html.replace("{{ files }}", &files)))
}

async fn login_index() -> Result<HttpResponse, Error> {
    let html = include_str!("../templates/password.html");

    Ok(HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, ContentType::html().try_into_value()?))
        .body(html))
}

#[post("/login")]
async fn login(form: web::Form<Login>, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let password = form.password.clone();

    if password == data.config.password {
        let redirect_html = include_str!("../templates/redirect.html");

        Ok(HttpResponse::Ok()
            .cookie(actix_web::cookie::Cookie::new("password", password))
            .append_header((header::CONTENT_TYPE, ContentType::html().try_into_value()?))
            .body(redirect_html.replace("{{ url }}", "/")))
    } else {
        let error_html = include_str!("../templates/error.html");

        Ok(HttpResponse::Ok()
            .append_header((header::CONTENT_TYPE, ContentType::html().try_into_value()?))
            .body(error_html.replace("{{ error }}", "Wrong password")))
    }
}
