use actix_files::Files;
use actix_web::{web, App, HttpServer, middleware::Logger};
use leptos::*;
use leptos_actix::{generate_route_list, LeptosRoutes};
use leptos::prelude::get_configuration;
use ui::App as LeptosApp;            // ← the public export from step 1

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // TODO: read Leptos conf from `Leptos.toml` or env
    let conf = get_configuration(None).unwrap();
    let routes = generate_route_list(|| view! { <LeptosApp/> });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(conf.clone()))
            .leptos_routes(routes.clone(), || view! { <LeptosApp/> })
            // static WASM + JS
            .service(
                Files::new("/pkg", "./target/site")
                     .show_files_listing()
            )
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

