// use actix_files::Files;
// use actix_web::{web, App, HttpServer, middleware::Logger};
// use leptos::*;
// use leptos_actix::{generate_route_list, LeptosRoutes};
// use leptos::prelude::get_configuration;
// use ui::App as LeptosApp;            // ← the public export from step 1
//
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     // TODO: read Leptos conf from `Leptos.toml` or env
//     let conf = get_configuration(None).unwrap();
//     let routes = generate_route_list(|| view! { <LeptosApp/> });
//
//     HttpServer::new(move || {
//         App::new()
//             .wrap(Logger::default())
//             .app_data(web::Data::new(conf.clone()))
//             // Serve static assets first
//             .service(
//                 Files::new("/assets", "./ui/assets")
//                     .use_last_modified(true)
//                     .prefer_utf8(true)
//             )
//             .service(
//                 Files::new("/assets", "../assets")
//                     .use_last_modified(true)
//                     .prefer_utf8(true)
//             )
//
//             // serve /assets/* from /home/bane/git/yus/assets/
//             .service(
//                 Files::new("/assets", "./assets")
//                     .use_last_modified(true)
//                     .prefer_utf8(true)
//             )
//
//             // serve /ui-assets/* from /home/bane/git/yus/ui/assets/
//             .service(
//                 Files::new("/ui-assets", "./ui/assets")
//                     .use_last_modified(true)
//                     .prefer_utf8(true)
//             )
//
//
//             // Serve Trunk-built WASM + JS
//             .service(
//                 Files::new("/pkg", "./target/site")
//                     .use_last_modified(true)
//                     .prefer_utf8(true)
//             )
//
//             // Only after static files are matched, fall back to Leptos routing
//             .leptos_routes(routes.clone(), || view! { <LeptosApp/> })
//     })
//     .bind(("127.0.0.1", 3000))?
//     .run()
//     .await
// }
//
//
// site/src/main.rs
// use actix_files::Files;
// use actix_web::{App, HttpServer, middleware::Logger};
//
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     env_logger::init();
//
//     HttpServer::new(|| {
//         App::new()
//             .wrap(Logger::default())
//             // serve the SPA bundle built by Trunk
//             .service(Files::new("/", "./dist").index_file("index.html"))
//             // (optional) API routes here
//     })
//     .bind(("127.0.0.1", 3000))?
//     .run()
//     .await
// }
//



// site/src/main.rs
// use actix_files::Files;
// use actix_web::{App, HttpServer, middleware::Logger, web};
// use std::path::PathBuf;
//
// async fn spa_fallback() -> actix_files::NamedFile {
//     // always return index.html so the SPA router can handle the URL
//     actix_files::NamedFile::open("./dist/index.html").unwrap()
// }
//
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     env_logger::init();
//
//     println!("hello");
//
//     HttpServer::new(|| {
//         App::new()
//             .wrap(Logger::default())
//             // serve the Trunk output (wasm, js, css, icons…)
//             .service(Files::new("/", "../dist").index_file("index.html").prefer_utf8(true))
//             // fallback for /anything/else
//             .default_service(web::get().to(spa_fallback))
//             // .route("/api/...", your JSON handlers here)
//     })
//     .bind(("127.0.0.1", 3000))?
//     .run()
//     .await
// }
//
//
use actix_files::Files;
use actix_web::{web, App, HttpServer, middleware::Logger};
use std::{env::current_dir, path::PathBuf};

async fn spa() -> actix_files::NamedFile {
    actix_files::NamedFile::open("./dist/index.html").unwrap()
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

