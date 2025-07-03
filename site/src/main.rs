use actix_files::{Files, NamedFile};
use actix_web::{web, App, HttpServer, middleware::Logger};
use std::env;
use dotenvy::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let projects_pat = env::var("PROJECTS_PAT").expect("PROJECTS_PAT missing");
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().unwrap();

    // TODO: github setup here

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(projects_pat.clone()))   // inject into handlers
            .service(Files::new("/pkg", "./dist"))
            .service(Files::new("/assets", "./dist/assets"))
            .service(
                Files::new("/", "./dist")
                    .index_file("index.html")
                    .default_handler(web::to(|| async {
                        // unmatched routes get index.html (SPA fallback)
                        NamedFile::open_async("./dist/index.html").await
                            .map_err(|e| {
                                eprintln!("Index fallback failed: {}", e);
                                actix_web::error::ErrorInternalServerError("Index load error")
                            })
                    })),
            )
            .default_service(web::get().to(index))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

async fn index() -> std::io::Result<actix_files::NamedFile> {
    actix_files::NamedFile::open_async("./dist/index.html").await
}
