use crate::database;
use actix_files::NamedFile;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Result};
use handlebars::Handlebars;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[get("/get_files")]
pub async fn get_files_json(
    data: web::Data<Mutex<Connection>>,
) -> actix_web::Result<impl Responder> {
    let mut listing: Vec<database::FileListing> =
        database::FileListing::read_all(data.get_ref()).unwrap_or(Vec::new());
    for item in listing.iter_mut() {
        if item.password.is_some() {
            item.password = Some("locked".to_owned());
        }
    }
    Ok(web::Json(listing))
}

#[get("/files")]
pub async fn get_files_main(
    db: web::Data<Mutex<Connection>>,
    hb: web::Data<Handlebars<'_>>,
) -> HttpResponse {
    let files = database::FileListing::read_all(db.as_ref()).unwrap();
    let body = hb.render("files", &files).unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

#[get("/files/{filename:.*}")]
pub async fn get_files(
    path: web::Path<String>,
    data: web::Data<Mutex<Connection>>,
) -> Result<NamedFile, std::io::Error> {
    /*
     * Files item in database
     * path: String
     * password: Option<String>
     * is_display: bool
     *
     */
    let page = database::FileListing::from_path(path.as_str(), data.get_ref());
    match page {
        None => Ok(NamedFile::open("src/pages/file_not_found.html")?),
        Some(page) => {
            if page.password.is_some() && page.password.unwrap() != "" {
                Ok(NamedFile::open("src/pages/enter_password.html")?)
            } else {
                Ok(NamedFile::open("files/".to_string() + &path.into_inner())?)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LockedFileRequest {
    password: String,
    path: String,
}

#[post("/locked_file")]
pub async fn get_locked_file(
    data: web::Data<Mutex<Connection>>,
    info: web::Json<LockedFileRequest>,
    request: HttpRequest,
) -> Result<impl Responder> {
    let page = database::FileListing::from_path(info.path.as_str(), data.get_ref());
    match page {
        None => Ok(HttpResponse::NotFound().finish()),
        Some(page) => {
            if page.password.is_some() && page.password.unwrap() == info.password {
                Ok(NamedFile::open("files/".to_string() + &info.path)?.into_response(&request))
            } else {
                Ok(HttpResponse::Forbidden().finish())
            }
        }
    }
}
