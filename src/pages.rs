use crate::database;
use actix_files::NamedFile;
use actix_web::{get, web, HttpResponse, Result};

use handlebars::Handlebars;
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

#[get("/src/pages/main.css")]
pub async fn get_css() -> Result<NamedFile, std::io::Error> {
    Ok(NamedFile::open("src/pages/main.css")?)
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
