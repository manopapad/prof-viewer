use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[cfg(not(target_arch = "wasm32"))]
use reqwest::blocking::{Client, ClientBuilder};
#[cfg(target_arch = "wasm32")]
use reqwest::{Client, ClientBuilder};

use url::Url;

use crate::{
    data::{EntryID, Initializer, SlotMetaTile, SlotTile, SummaryTile, TileID},
    deferred_data::DeferredDataSource,
    http::fetch::ProfResponse,
    logging::*,
    queue::queue::{ProcessType, Work},
    timestamp::Interval,
};

use crate::http::fetch::fetch;

use super::schema::{FetchRequest, FetchTilesRequest};

pub struct HTTPQueueDataSource {
    pub url: Url,
    pub client: Client,
    pub queue: Arc<Mutex<Vec<Work>>>,
    pub info: Option<Initializer>,
    pub interval: Interval,
    fetch_tiles_cache: BTreeMap<EntryID, Vec<TileID>>,
    fetch_summary_tiles_cache: Vec<SummaryTile>,
    fetch_slot_tiles_cache: Vec<SlotTile>,
    fetch_slot_meta_tiles_cache: Vec<SlotMetaTile>,
}

impl HTTPQueueDataSource {
    pub fn new(url: Url) -> Self {
        // log("INIT HTTPQueueDataSource");
        let queue: std::sync::Arc<std::sync::Mutex<Vec<Work>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        Self {
            url,
            client: ClientBuilder::new().build().unwrap(),
            queue,
            info: None,
            interval: Interval::default(),
            fetch_tiles_cache: BTreeMap::new(),
            fetch_summary_tiles_cache: Vec::new(),
            fetch_slot_meta_tiles_cache: Vec::new(),
            fetch_slot_tiles_cache: Vec::new(),
        }
    }

    // empty queue and add results to respective caches
    fn process_queue(&mut self) {
        // log("process_queue");
        let mut q = self.queue.lock().unwrap();

        for work in q.iter() {
            match work.process_type {
                ProcessType::FetchSlotMetaTile => {
                    // deserialize work.data into SlotMetaTile
                    let smt = serde_json::from_str::<SlotMetaTile>(&work.data).unwrap();
                    // add to cache or create new vector

                    self.fetch_slot_meta_tiles_cache.push(smt.clone());
                }
                ProcessType::FetchSlotTile => {
                    // deserialize work.data into SlotTile
                    let st = serde_json::from_str::<SlotTile>(&work.data).unwrap();
                    // add to cache
                    self.fetch_slot_tiles_cache.push(st.clone());
                }

                ProcessType::FetchTiles => {
                    // deserialize work.data into Vec<TileID>
                    let tiles = serde_json::from_str::<Vec<TileID>>(&work.data).unwrap();
                    // add to cache
                    self.fetch_tiles_cache
                        .entry(work.entry_id.clone())
                        .or_insert(tiles.clone())
                        .extend(tiles.clone());
                }
                ProcessType::FetchSummaryTile => {
                    // deserialize work.data into SummaryTile
                    let st = serde_json::from_str::<SummaryTile>(&work.data).unwrap();
                    // add to cache
                    self.fetch_summary_tiles_cache.push(st.clone());
                }
                ProcessType::INTERVAL => {
                    // deserialize work.data into Interval
                    let interval = serde_json::from_str::<Interval>(&work.data).unwrap();
                    // add to cache
                    self.interval = interval;
                }
                ProcessType::FetchInfo => {
                    // deserialize work.data into EntryInfo
                    // console_log!("found fetch info in queue");
                    let info: Initializer =
                        serde_json::from_str::<Initializer>(&work.data).unwrap();
                    // add to cache
                    self.info = Some(info);
                }
            }
        }
        // empty queue
        q.clear(); // ?
    }

    fn queue_work(&mut self, work: Work) {
        // log("queue_work");
        let _work = work.clone();
        let url = match work.process_type {
            ProcessType::FetchSlotMetaTile => self
                .url
                .join("/slot_meta_tile")
                .expect("Invalid URL with /slot_meta_tile"),
            ProcessType::FetchSlotTile => self
                .url
                .join("/slot_tile")
                .expect("Invalid URL with /slot_tile"),
            ProcessType::FetchTiles => self.url.join("/tiles").expect("Invalid URL with /tiles"),
            ProcessType::FetchSummaryTile => self
                .url
                .join("/summary_tile")
                .expect("Invalid URL with /summary_tile"),
            ProcessType::INTERVAL => self
                .url
                .join("/interval")
                .expect("Invalid URL with /interval"),
            ProcessType::FetchInfo => self.url.join("/info").expect("Invalid URL with /info"),
        };

        let body = match work.process_type {
            ProcessType::FetchSlotMetaTile => serde_json::to_string(&FetchRequest {
                entry_id: work.entry_id.clone(),
                tile_id: work.tile_id.unwrap(),
            })
            .unwrap(),
            ProcessType::FetchSlotTile => serde_json::to_string(&FetchRequest {
                entry_id: work.entry_id.clone(),
                tile_id: work.tile_id.unwrap(),
            })
            .unwrap(),
            ProcessType::FetchTiles => serde_json::to_string(&FetchTilesRequest {
                entry_id: work.entry_id.clone(),
                interval: work.interval.unwrap(),
            })
            .unwrap(),
            ProcessType::FetchSummaryTile => serde_json::to_string(&FetchRequest {
                entry_id: work.entry_id.clone(),
                tile_id: work.tile_id.unwrap(),
            })
            .unwrap(),
            ProcessType::INTERVAL => "".to_string(),
            ProcessType::FetchInfo => "".to_string(),
        };
        let request = self
            .client
            .post(url)
            .header("Accept", "*/*")
            .header("Content-Type", "javascript/json;")
            .body(body);

        let queue = self.queue.clone();

        fetch(request, move |result: Result<ProfResponse, String>| {
            // deserialize response into a vector of TileIDs
            let work = Work {
                entry_id: work.entry_id.clone(),
                tile_id: work.tile_id,
                tile_ids: _work.tile_ids.clone(),
                interval: work.interval,
                data: result.unwrap().body,
                process_type: work.process_type,
            };

            queue.lock().unwrap().push(work);
        });
    }
}

impl DeferredDataSource for HTTPQueueDataSource {
    fn fetch_info(&mut self) {
        self.process_queue();

        let work = Work {
            entry_id: EntryID::root(),
            tile_id: None,
            tile_ids: None,
            interval: None,
            data: "".to_string(),
            process_type: ProcessType::FetchInfo,
        };
        self.queue_work(work);
        // console_log!("added fetch_info to queue");
    }

    fn get_info(&mut self) -> Option<Initializer> {
        // console_log!("checking get_info");
        self.process_queue();
        self.info.clone()
    }

    fn fetch_tiles(&mut self, entry_id: EntryID, request_interval: Interval) {
        self.process_queue();
        // queue work
        let work = Work {
            entry_id: entry_id.clone(),
            tile_id: None,
            tile_ids: None,
            interval: Some(request_interval),
            data: "".to_string(),
            process_type: ProcessType::FetchTiles,
        };
        self.queue_work(work);
    }

    fn get_tiles(&mut self, entry_id: EntryID) -> Vec<TileID> {
        self.process_queue();
        if let Some(tiles) = self.fetch_tiles_cache.get(&(entry_id.clone())) {
            return tiles.to_vec();
        } else {
            return vec![];
        }
    }

    fn fetch_summary_tile(&mut self, entry_id: EntryID, tile_id: TileID) {
        // queue work
        self.process_queue();
        let work = Work {
            entry_id: entry_id.clone(),
            tile_id: Some(tile_id),
            interval: None,
            tile_ids: None,
            data: "".to_string(),
            process_type: ProcessType::FetchSummaryTile,
        };
        self.queue_work(work);
    }

    fn get_summary_tiles(&mut self) -> Vec<SummaryTile> {
        self.process_queue();

        let tiles = self.fetch_summary_tiles_cache.clone();
        self.fetch_summary_tiles_cache.clear();
        tiles
    }
    fn fetch_slot_tile(&mut self, entry_id: EntryID, tile_id: TileID) {
        self.process_queue();
        // queue work
        let work = Work {
            entry_id: entry_id.clone(),
            tile_id: Some(tile_id),
            interval: None,
            tile_ids: None,
            data: "".to_string(),
            process_type: ProcessType::FetchSlotTile,
        };
        self.queue_work(work);
    }

    fn get_slot_tile(&mut self) -> Vec<SlotTile> {
        self.process_queue();

        let tiles = self.fetch_slot_tiles_cache.clone();
        self.fetch_slot_tiles_cache.clear();
        tiles
    }

    fn fetch_slot_meta_tile(&mut self, entry_id: EntryID, tile_id: TileID) {
        self.process_queue();
        // check cache

        // queue work
        let work = Work {
            entry_id: entry_id.clone(),
            tile_id: Some(tile_id),
            tile_ids: None,
            interval: None,
            data: "".to_string(),
            process_type: ProcessType::FetchSlotMetaTile,
        };
        self.queue_work(work);
    }

    fn get_slot_meta_tile(&mut self) -> Vec<SlotMetaTile> {
        self.process_queue();

        let tiles = self.fetch_slot_meta_tiles_cache.clone();
        self.fetch_slot_meta_tiles_cache.clear();
        tiles
    }
}
