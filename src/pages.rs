use crate::database;
use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest, HttpResponse, Result};

use handlebars::Handlebars;
use lazy_static::lazy_static;
use rusqlite::Connection;
use std::{
    fs::{self, DirEntry},
    path::Path,
    sync::Mutex,
};

#[get("/")]
pub async fn index(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    crate::template_to_response(&hb, "index")
}

fn recurse_directories(dir: &Path, cb: &mut impl FnMut(&DirEntry)) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            recurse_directories(&path, cb)?;
        } else {
            cb(&entry)
        }
    }
    Ok(())
}

lazy_static! {
    static ref STATIC_FILES: std::collections::HashSet<String> = {
        let mut set = std::collections::HashSet::new();
        recurse_directories(Path::new("src/static"), &mut |entry: &DirEntry| {
            let path = entry.path();
            let path = path.to_str().unwrap();
            set.insert((&path[path.find("src/static").unwrap()..]).to_owned());
        })
        .unwrap();
        set
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
    hb: web::Data<Handlebars<'_>>,
) -> Result<HttpResponse, std::io::Error> {
    let link = database::LinkListing::from_path(path.as_str(), data.get_ref());
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(match link {
            None => hb.render("file_not_found", &0).unwrap(),
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
pub async fn get_sitemap(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    crate::template_to_response(&hb, "sitemap")
}

#[get("/projects/{project}")]
pub async fn get_projects(
    path: web::Path<(String,)>,
    hb: web::Data<Handlebars<'_>>,
) -> Option<HttpResponse> {
    Some(HttpResponse::Ok().content_type("text/html").body(
        match hb.render(("projects".to_owned() + &"/" + &path.0).as_str(), &0) {
            Err(_) => hb.render("file_not_found", &0).ok()?,
            Ok(file) => file,
        },
    ))
}
