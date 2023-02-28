use crate::data::{DataSource, EntryID, EntryInfo, TileID};
use crate::timestamp::Interval;

use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder, Result,
};
use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};

pub struct AppState {
    pub data_source: Mutex<Box<dyn DataSource + Sync + Send + 'static>>,
}

pub struct DataSourceHTTPServer {
    pub port: u16,
    pub host: String,
    pub state: AppState,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchRequest {
    pub entry_id: EntryID,
    pub tile_id: TileID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchTilesRequest {
    pub entry_id: EntryID,
    pub interval: Interval,
}

impl DataSourceHTTPServer {
    // new function

    pub fn new(
        port: u16,
        host: String,
        state: Box<dyn DataSource + Sync + Send + 'static>,
    ) -> Self {
        // let state = Data::from(Arc::new(data));
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
        let mut e = String::new();
        match source.fetch_info() {
            EntryInfo::Panel { short_name, .. } => {
                e = short_name.clone();
            }
            _ => e = "hello".to_string(),
        }

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
        let to_ret = source.request_tiles(entry_id, request_interval);
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

    #[actix_web::main]
    pub async fn create_server(
        self,
        // data: impl DataSource + Send + Sync + 'static,
    ) -> std::io::Result<()> {
        // let app_state = AppState {
        //     data_source: Mutex::new(Box::new(data)),
        // };

        let state = Data::from(Arc::new(self.state));
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default())
                .wrap(middleware::Compress::default())
                .app_data(state.clone())
                .route("/entry", web::get().to(Self::get_entry_name))
                .route("/info", web::get().to(Self::fetch_info))
                .route("/interval", web::get().to(Self::interval))
                .route("/tiles", web::get().to(Self::fetch_tiles))
                .route("/slot_meta_tile", web::get().to(Self::fetch_slot_meta_tile))
                .route("/slot_tile", web::get().to(Self::fetch_slot_tile))
                .route("/summary_tile", web::get().to(Self::fetch_summary_tile))
        })
        .bind((self.host.as_str(), self.port))?
        .run()
        .await
    }
}
