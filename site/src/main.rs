use actix_files::Files;
use actix_web::HttpRequest;
use actix_web::{web, App, HttpServer, middleware::Logger};
use std::{env::current_dir, path::PathBuf};
use std::env;


async fn spa() -> actix_files::NamedFile {
    actix_files::NamedFile::open("../dist/index.html").unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // print CWD for debug
    println!("CWD = {:?}", std::env::current_dir());

    let port: u16 = env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(Files::new("/",        "./dist").index_file("index.html"))
            .service(Files::new("/assets",  "./ui/assets"))
            .default_service(web::get().to(|| async {
            match actix_files::NamedFile::open_async("./dist/index.html").await {
                Ok(file) => Ok(file),
                Err(e) => {
                    eprintln!("shit: {:?}", e);
                    Err(e)
                }
            }
        }))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
