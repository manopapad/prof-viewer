use crate::data::{DataSource, EntryInfo};

use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder, Result, http,
};
use actix_cors::Cors;

use std::sync::{Arc, Mutex};

use super::schema::{FetchMultipleRequest, FetchRequest, FetchTilesRequest};

// dyn DataSource + Sync + Send + 'static> from
// https://stackoverflow.com/questions/65645622/how-do-i-pass-a-trait-as-application-data-to-actix-web
// to enable passing a datasource between threads
pub struct AppState {
    pub data_source: Mutex<Box<dyn DataSource + Sync + Send + 'static>>,
}

pub struct DataSourceHTTPServer {
    pub port: u16,
    pub host: String,
    pub state: AppState,
}

impl DataSourceHTTPServer {
    pub fn new(
        port: u16,
        host: String,
        state: Box<dyn DataSource + Sync + Send + 'static>,
    ) -> Self {
        Self {
            port,
            host,
            state: AppState {
                data_source: Mutex::new(state),
            },
        }
    }
    async fn get_entry_name(data: web::Data<AppState>) -> impl Responder {
        let mutex = &data.data_source;
        let mut source = mutex.lock().unwrap();
        let e = match source.fetch_info() {
            EntryInfo::Panel { short_name, .. } => short_name.clone(),
            _ => "hello".to_string(),
        };

        HttpResponse::Ok().body(e)
    }

    async fn fetch_info(data: web::Data<AppState>) -> Result<impl Responder> {
        let mutex = &data.data_source;
        let mut source = mutex.lock().unwrap();
        let to_ret = source.fetch_info().clone();
        Ok(web::Json(to_ret))
    }

    async fn interval(data: web::Data<AppState>) -> Result<impl Responder> {
        let mutex = &data.data_source;
        let mut source = mutex.lock().unwrap();
        let to_ret = source.interval();
        Ok(web::Json(to_ret))
    }

    async fn fetch_tiles(
        info: web::Json<FetchTilesRequest>,
        data: web::Data<AppState>,
    ) -> Result<impl Responder> {
        let mutex = &data.data_source;
        let mut source = mutex.lock().unwrap();

        let entry_id = &info.entry_id;
        let request_interval = info.interval;
        let to_ret = source.fetch_tiles(entry_id, request_interval);
        Ok(web::Json(to_ret))
    }

    async fn fetch_slot_meta_tile(
        info: web::Json<FetchRequest>,
        data: web::Data<AppState>,
    ) -> Result<impl Responder> {
        let mutex = &data.data_source;
        let mut source = mutex.lock().unwrap();

        let entry_id = &info.entry_id;
        let tile_id = info.tile_id;
        let to_ret = source.fetch_slot_meta_tile(entry_id, tile_id);
        Ok(web::Json(to_ret))
    }

    async fn fetch_slot_tile(
        info: web::Json<FetchRequest>,
        data: web::Data<AppState>,
    ) -> Result<impl Responder> {
        let mutex = &data.data_source;
        let mut source = mutex.lock().unwrap();

        let entry_id = &info.entry_id;
        let tile_id = info.tile_id;
        let to_ret = source.fetch_slot_tile(entry_id, tile_id);
        Ok(web::Json(to_ret))
    }


    async fn fetch_summary_tile(
        info: web::Json<FetchRequest>,
        data: web::Data<AppState>,
    ) -> Result<impl Responder> {
        let mutex = &data.data_source;
        let mut source = mutex.lock().unwrap();

        let entry_id = &info.entry_id;
        let tile_id = info.tile_id;
        let to_ret = source.fetch_summary_tile(entry_id, tile_id);
        Ok(web::Json(to_ret))
    }


    async fn init(data: web::Data<AppState>) -> Result<impl Responder> {
        let mutex = &data.data_source;
        let mut source = mutex.lock().unwrap();
        let to_ret = source.init();
        Ok(web::Json(to_ret))
    }

    #[actix_web::main]
    pub async fn create_server(self) -> std::io::Result<()> {
        let state = Data::from(Arc::new(self.state));
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();
        HttpServer::new(move || {
            let cors = Cors::default()
            .send_wildcard()
            .allow_any_origin()
            // .allowed_origin("All")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
            App::new()
                .wrap(middleware::Logger::default())
                .wrap(middleware::Compress::default())
                .wrap(cors)
                .app_data(state.clone())
                .route("/entry", web::post().to(Self::get_entry_name))
                .route("/info", web::post().to(Self::fetch_info))
                .route("/interval", web::post().to(Self::interval))
                .route("/tiles", web::post().to(Self::fetch_tiles))
                .route(
                    "/slot_meta_tile",
                    web::post().to(Self::fetch_slot_meta_tile),
                )
                .route("/slot_tile", web::post().to(Self::fetch_slot_tile))
                .route("/summary_tile", web::post().to(Self::fetch_summary_tile))
                .route("/init", web::post().to(Self::init))
        })
        .bind((self.host.as_str(), self.port))?
        .run()
        .await
    }
}
