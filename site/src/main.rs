use actix_files::Files;
use actix_web::{web, App, HttpServer, middleware::Logger};
use std::{env::current_dir, path::PathBuf};

async fn spa() -> actix_files::NamedFile {
    actix_files::NamedFile::open("../dist/index.html").unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));        // = site/
    let cwd = current_dir();
    println!("{:?}", cwd);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            // ① serve the SPA bundle built by Trunk
            .service(Files::new("/",        "../dist").index_file("index.html"))
            // ② serve top-level static assets
            .service(Files::new("/assets",  root.join("../assets")))
            // ③ fallback -> SPA for any other path
            .default_service(web::get().to(spa))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

