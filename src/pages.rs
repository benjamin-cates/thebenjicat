use crate::database;
use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest, HttpResponse, Result};

use handlebars::Handlebars;
use lazy_static::lazy_static;
use rusqlite::Connection;
use std::io::Read;
use std::sync::Mutex;

fn file_to_string(path: &str) -> Result<String, std::io::Error> {
    let file = NamedFile::open(path)?;
    let mut contents = String::new();
    file.file().read_to_string(&mut contents)?;
    Ok(contents)
}

#[get("/")]
pub async fn index() -> Result<NamedFile, std::io::Error> {
    Ok(NamedFile::open("src/index.html")?)
}

lazy_static! {
    static ref STATIC_FILES: std::collections::HashSet<&'static str> = {
        [
            "src/static/images/interactive_em.jpg",
            "src/static/images/matrix_assistant.jpg",
            "src/static/images/pfp.jpg",
            "src/static/images/piha.jpg",
            "src/static/images/topo_map.jpg",
            "src/static/style/main.css",
        ]
        .into_iter()
        .collect()
    };
}

#[get("/static/{folder}/{file}")]
pub async fn get_static_file(
    path: web::Path<(String, String)>,
    request: HttpRequest,
) -> Result<HttpResponse, std::io::Error> {
    let full_path = "src/static/".to_owned() + path.0.as_ref() + &"/" + path.1.as_ref();
    if STATIC_FILES.get(full_path.as_str()).is_none() {
        return Ok(HttpResponse::Forbidden().finish());
    }
    Ok(
        match NamedFile::open("src/static/".to_owned() + path.0.as_ref() + &"/" + path.1.as_ref()) {
            Ok(file) => file.into_response(&request),
            Err(_) => HttpResponse::NotFound().finish(),
        },
    )
}

#[get("/links/{link}")]
pub async fn get_links(
    path: web::Path<String>,
    data: web::Data<Mutex<Connection>>,
) -> Result<HttpResponse, std::io::Error> {
    let link = database::LinkListing::from_path(path.as_str(), data.get_ref());
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(match link {
            None => file_to_string("src/pages/file_not_found.html")?,
            Some(link) => format!(
                "<!DOCTYPE HTML><html><head><meta http-equiv=\"refresh\" content=\"0; URL={}\"/></head></html>",
                link.destination
            ),
        }))
}

#[get("/links")]
pub async fn get_links_main(
    hb: web::Data<Handlebars<'_>>,
    db: web::Data<Mutex<Connection>>,
) -> Result<HttpResponse, std::io::Error> {
    let links = database::LinkListing::read_all(db.as_ref()).unwrap();
    let body = hb.render("links", &links).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[get("/sitemap")]
pub async fn get_sitemap() -> Result<NamedFile, std::io::Error> {
    Ok(NamedFile::open("src/pages/sitemap.html")?)
}

#[get("/projects/{project}")]
pub async fn get_projects(path: web::Path<(String,)>) -> Result<NamedFile, std::io::Error> {
    let file = NamedFile::open("src/pages/projects/".to_owned() + &path.0 + ".html");
    match file {
        Err(_) => NamedFile::open("src/pages/file_not_found.html"),
        Ok(f) => Ok(f),
    }
}
