use crate::AdminPassword;
use actix_web::web::{self, Bytes};
use actix_web::HttpResponse;
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Mutex;

pub fn admin_scope() -> actix_web::Scope {
    web::scope("/admin")
        .route("/index", web::get().to(get_admin_page))
        .route("/add_file", web::post().to(add_file))
        .route("/add_link", web::post().to(add_link))
        .route("/remove_link", web::post().to(remove_link))
        .route("/remove_file", web::post().to(remove_file))
}

pub async fn get_admin_page(hb: web::Data<handlebars::Handlebars<'_>>) -> HttpResponse {
    crate::template_to_response(&hb, "admin")
}

async fn add_file(
    bytes: Bytes,
    admin_password_real: web::Data<AdminPassword>,
    db: web::Data<Mutex<Connection>>,
) -> HttpResponse {
    let read_nth = |n: usize| {
        bytes
            .split(|ch| *ch == b',')
            .nth(n)
            .map(|string| std::str::from_utf8(string).ok())
            .flatten()
    };
    let admin_password_claim = match read_nth(0) {
        Some(pass) => pass,
        None => return HttpResponse::BadRequest().finish(),
    };
    let name = match read_nth(1) {
        Some(name) => name,
        None => return HttpResponse::BadRequest().finish(),
    };
    let password = match read_nth(2) {
        Some(password) => password,
        None => return HttpResponse::BadRequest().finish(),
    };
    if admin_password_claim != admin_password_real.0 {
        return HttpResponse::Forbidden().finish();
    }
    let file_list = crate::database::FileListing {
        id: 0,
        path: name.to_owned(),
        password: if password.len() == 0 {
            None
        } else {
            Some(password.to_owned())
        },
    };
    let db_result = file_list.push_to_db(&db);
    if db_result.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    let third_comma = admin_password_claim.len() + 1 + name.len() + 1 + password.len() + 1;
    return match std::fs::write("files/".to_owned() + &name, &bytes[third_comma..]) {
        Err(_) => {
            let _ = crate::database::FileListing {
                id: 0,
                path: name.to_owned(),
                password: None,
            }
            .remove_from_db(&db);
            HttpResponse::InternalServerError().finish()
        }
        Ok(()) => HttpResponse::Ok().finish(),
    };
}
#[derive(Deserialize)]
struct AddLink {
    admin_password: String,
    name: String,
    path: String,
    url: String,
}
async fn add_link(
    json: web::Json<AddLink>,
    db: web::Data<Mutex<Connection>>,
    admin_password_real: web::Data<AdminPassword>,
) -> HttpResponse {
    if admin_password_real.get_ref().0 != json.admin_password.as_str() {
        return HttpResponse::Forbidden().finish();
    }
    let json = json.into_inner();
    match (crate::database::LinkListing {
        id: 0,
        name: json.name,
        path: json.path,
        destination: json.url,
    })
    .push_to_db(&db)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
#[derive(Deserialize)]
struct RemoveLink {
    admin_password: String,
    path: String,
}
async fn remove_link(
    json: web::Json<RemoveLink>,
    db: web::Data<Mutex<Connection>>,
    admin_password_real: web::Data<AdminPassword>,
) -> HttpResponse {
    if admin_password_real.get_ref().0 != json.admin_password.as_str() {
        return HttpResponse::Forbidden().finish();
    }
    let json = json.into_inner();
    match (crate::database::LinkListing {
        id: 0,
        name: "".to_owned(),
        path: json.path,
        destination: "".to_owned(),
    })
    .remove_from_db(&db)
    {
        Ok(1) => HttpResponse::Ok().finish(),
        Ok(_) => HttpResponse::NotFound().finish(),
        Err(err) => HttpResponse::InternalServerError().body(format!("{:?}", err)),
    }
}
#[derive(Deserialize)]
struct RemoveFile {
    admin_password: String,
    name: String,
}
async fn remove_file(
    json: web::Json<RemoveFile>,
    db: web::Data<Mutex<Connection>>,
    admin_password_real: web::Data<AdminPassword>,
) -> HttpResponse {
    if admin_password_real.get_ref().0 != json.admin_password.as_str() {
        return HttpResponse::Forbidden().finish();
    }
    let json = json.into_inner();
    match (crate::database::FileListing {
        id: 0,
        password: None,
        path: json.name.clone(),
    })
    .remove_from_db(&db)
    {
        Ok(1) => HttpResponse::Ok().finish(),
        Ok(_) => return HttpResponse::NotFound().finish(),
        Err(err) => return HttpResponse::InternalServerError().body(format!("{:?}", err)),
    };
    match std::fs::remove_file("files/".to_owned() + &json.name) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().body(format!("{err:?}")),
    }
}
