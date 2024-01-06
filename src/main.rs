use actix_web::{web::Data, App, HttpResponse, HttpServer};
use handlebars::{DirectorySourceOptions, Handlebars};
mod database;
mod files;
mod pages;

pub(crate) fn template_to_response(hb: &Data<Handlebars<'_>>, path: &str) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(hb.render(path, &0).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut hb: Handlebars = Handlebars::new();
    hb.register_templates_directory(
        "src/pages",
        DirectorySourceOptions {
            tpl_extension: ".html".to_owned(),
            hidden: false,
            temporary: false,
        },
    )
    .unwrap();

    std::env::set_var("RUST_LOG", "actix_web=trace");
    env_logger::init();
    let db = database::open_connection();
    let hb_data = Data::new(hb);
    let db_data = Data::new(db);
    println!("Hosting on localhost:8000");
    HttpServer::new(move || {
        App::new()
            .service(pages::index)
            .service(pages::get_links)
            .service(pages::get_projects)
            .service(pages::get_sitemap)
            .service(pages::get_static_file)
            .service(pages::get_links_main)
            .service(files::get_files)
            .service(files::get_files_json)
            .service(files::get_files_main)
            .service(files::get_locked_file)
            .app_data(Data::clone(&db_data))
            .app_data(Data::clone(&hb_data))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await?;
    return Ok(());
}
