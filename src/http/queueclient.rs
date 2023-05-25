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
    console_log,
    data::{EntryID, Initializer, SlotMetaTile, SlotTile, SummaryTile, TileID},
    deferred_data::DeferredDataSource,
    http::fetch::ProfResponse,
    queue::queue::{Data, Work},
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

        for work in q.drain(..) {
            match work {
                Work::FetchSlotMetaTile(_, _, data) => {
                    if let Data::Ready(smt) = data {
                        // add to cache
                        self.fetch_slot_meta_tiles_cache.push(smt);
                    }
                }
                Work::FetchSlotTile(_, _, data) => {
                    if let Data::Ready(st) = data {
                        // add to cache
                        self.fetch_slot_tiles_cache.push(st);
                    }
                }

                Work::FetchTiles(entry_id, _, data) => {
                    if let Data::Ready(tiles) = data {
                        self.fetch_tiles_cache
                            .entry(entry_id.clone())
                            .or_insert(tiles.clone())
                            .extend(tiles.clone());
                    }
                }
                Work::FetchSummaryTile(_, _, data) => {
                    // deserialize work.data into SummaryTile
                    if let Data::Ready(st) = data {
                        // add to cache
                        self.fetch_summary_tiles_cache.push(st);
                    }
                }
                Work::FetchInterval(data) => {
                    // deserialize work.data into Interval
                    if let Data::Ready(interval) = data {
                        // add to cache
                        self.interval = interval;
                    }
                }
                Work::FetchInfo(data) => {
                    if let Data::Ready(info) = data {
                        // add to cache
                        self.info = Some(info);
                    }
                }
            }
        }
    }

    fn queue_work(&mut self, mut work: Work) {
        // log("queue_work");

        // create a mutable reference to work variable

        let url: Url = match &work {
            Work::FetchSlotMetaTile(_entry_id, _tile_id, _data) => self
                .url
                .join("/slot_meta_tile")
                .expect("Invalid URL with /slot_meta_tile"),
            Work::FetchSlotTile(_entry_id, _tile_id, _data) => self
                .url
                .join("/slot_tile")
                .expect("Invalid URL with /slot_tile"),
            Work::FetchTiles(_entry_id, _interval, _data) => {
                self.url.join("/tiles").expect("Invalid URL with /tiles")
            }
            Work::FetchSummaryTile(_entry_id, _tile_id, _data) => self
                .url
                .join("/summary_tile")
                .expect("Invalid URL with /summary_tile"),
            Work::FetchInterval(_) => self
                .url
                .join("/interval")
                .expect("Invalid URL with /interval"),
            Work::FetchInfo(_) => self.url.join("/info").expect("Invalid URL with /info"),
        };

        let body = match &work {
            Work::FetchSlotMetaTile(entry_id, tile_id, _data) => {
                serde_json::to_string(&FetchRequest {
                    entry_id: entry_id.clone(),
                    tile_id: *tile_id,
                })
                .unwrap()
            }
            Work::FetchSlotTile(entry_id, tile_id, _data) => serde_json::to_string(&FetchRequest {
                entry_id: entry_id.clone(),
                tile_id: *tile_id,
            })
            .unwrap(),
            Work::FetchTiles(entry_id, interval, _data) => {
                serde_json::to_string(&FetchTilesRequest {
                    entry_id: entry_id.clone(),
                    interval: *interval,
                })
                .unwrap()
            }
            Work::FetchSummaryTile(entry_id, tile_id, _data) => {
                serde_json::to_string(&FetchRequest {
                    entry_id: entry_id.clone(),
                    tile_id: *tile_id,
                })
                .unwrap()
            }
            Work::FetchInterval(_) => "".to_string(),
            Work::FetchInfo(_) => "".to_string(),
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

            // process response as data type in work

            let result = result.unwrap();
            let text = &result.body;

            match work {
                Work::FetchSlotMetaTile(entry_id, tile_id, _data) => {
                    work = Work::FetchSlotMetaTile(
                        entry_id,
                        tile_id,
                        Data::Ready(serde_json::from_str::<SlotMetaTile>(text).unwrap()),
                    );
                }
                Work::FetchSlotTile(entry_id, tile_id, _data) => {
                    work = Work::FetchSlotTile(
                        entry_id,
                        tile_id,
                        Data::Ready(serde_json::from_str::<SlotTile>(text).unwrap()),
                    );
                }
                Work::FetchTiles(entry_id, interval, _data) => {
                    work = Work::FetchTiles(
                        entry_id,
                        interval,
                        Data::Ready(serde_json::from_str::<Vec<TileID>>(text).unwrap()),
                    );
                }
                Work::FetchSummaryTile(entry_id, tile_id, _data) => {
                    work = Work::FetchSummaryTile(
                        entry_id,
                        tile_id,
                        Data::Ready(serde_json::from_str::<SummaryTile>(text).unwrap()),
                    );
                }
                Work::FetchInterval(_data) => {
                    work = Work::FetchInterval(Data::Ready(
                        serde_json::from_str::<Interval>(text).unwrap(),
                    ));
                }
                Work::FetchInfo(_data) => {
                    work = Work::FetchInfo(Data::Ready(
                        serde_json::from_str::<Initializer>(text).unwrap(),
                    ));
                }
            }

            queue.lock().unwrap().push(work.clone());
        });
    }
}

impl DeferredDataSource for HTTPQueueDataSource {
    fn fetch_info(&mut self) {
        self.process_queue();

        let work = Work::FetchInfo(Data::Requested);
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
        let work = Work::FetchTiles(entry_id, request_interval, Data::Requested);
        self.queue_work(work);
    }

    fn get_tiles(&mut self, entry_id: EntryID) -> Vec<TileID> {
        self.process_queue();
        if let Some(tiles) = self.fetch_tiles_cache.get(&entry_id) {
            tiles.to_vec()
        } else {
            vec![]
        }
    }

    fn fetch_summary_tile(&mut self, entry_id: EntryID, tile_id: TileID) {
        // queue work
        self.process_queue();

        let work = Work::FetchSummaryTile(entry_id, tile_id, Data::Requested);
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

        let work = Work::FetchSlotTile(entry_id, tile_id, Data::Requested);
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
        let work = Work::FetchSlotMetaTile(entry_id, tile_id, Data::Requested);
        self.queue_work(work);
    }

    fn get_slot_meta_tile(&mut self) -> Vec<SlotMetaTile> {
        self.process_queue();

        let tiles = self.fetch_slot_meta_tiles_cache.clone();
        self.fetch_slot_meta_tiles_cache.clear();
        tiles
    }
}
