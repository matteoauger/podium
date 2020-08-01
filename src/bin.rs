extern crate podium_lib;
use podium_lib::config::{get_config, AppConfig};
use podium_lib::contracts::app_state::*;
use podium_lib::routes::search;
use podium_lib::tantivy_process::{start_tantivy, tantivy_init, TantivyConfig};

#[macro_use]
extern crate log;

extern crate clap;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

use app_dirs::*;
use config::*;

const APP_INFO: AppInfo = AppInfo {
    name: "Podium",
    author: "Teodor Voinea",
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = get_config();
    // dbg!(&config);
    simple_logger::init_with_level(config.verbosity).unwrap();
    let local = tokio::task::LocalSet::new();

    // Get or create settings
    let settings = get_or_create_settings(&config);

    let (searcher, mut tantivy_wrapper) = tantivy_init(&settings).unwrap();

    let _tantivy_thread = tokio::spawn(async move {
        start_tantivy(&settings, &mut tantivy_wrapper)
            .await
            .unwrap();
    });

    let sys = actix_rt::System::run_in_tokio("server", &local);

    let app_state = web::Data::new(AppState { searcher: searcher });

    let server_res = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::new() // <- Construct CORS middleware builder
                    .send_wildcard()
                    .finish(),
            )
            .app_data(app_state.clone())
            .configure(search::server_config)
    })
    .bind(format!("127.0.0.1:{}", config.port))?
    .run()
    .await?;

    sys.await?;

    Ok(server_res)

    // if tantivy_thread.unwrap().join().is_err() {
    //     error!("Failed to join tantivy thread");
    // }
}

fn get_or_create_settings(app_config: &AppConfig) -> TantivyConfig {
    let index_path = app_dir(AppDataType::UserData, &APP_INFO, "index").unwrap();
    info!("Using index file in: {:?}", index_path);

    let state_path = app_dir(AppDataType::UserData, &APP_INFO, "state").unwrap();
    let mut initial_processing_file = state_path.clone();
    initial_processing_file.push("initial_processing");

    TantivyConfig {
        index_path: index_path,
        scan_directories: app_config.scan_directories.clone(),
        initial_processing_file: initial_processing_file,
    }
}
