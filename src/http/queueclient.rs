use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use reqwest::header::Entry;
use serde::{Deserialize, Serialize};

use crate::{
    data::{
        DataSource, EntryID, EntryInfo, Initializer, SlotMetaTile, SlotTile, SummaryTile, TileID,
    },
    logging::*,
    queue::queue::{ProcessType, Work},
    timestamp::Interval,
};
use ehttp::{self, headers, Request};

use super::schema::{FetchMultipleRequest, FetchRequest, FetchTilesRequest};

pub struct HTTPQueueDataSource {
    pub host: String,
    pub port: u16,
    pub client: reqwest::Client,
    pub queue: Arc<Mutex<Vec<Work>>>,
    pub fetch_info: EntryInfo,
    pub initializer: Initializer,
    pub interval: Interval,
    request_tiles_cache: BTreeMap<(EntryID, Interval), Vec<TileID>>,
    fetch_summary_tile_cache: BTreeMap<(EntryID, TileID), SummaryTile>,
    fetch_summary_tiles_cache: BTreeMap<(EntryID, Vec<TileID>), Vec<SummaryTile>>,
    fetch_slot_tile_cache: BTreeMap<(EntryID, TileID), SlotTile>,
    fetch_slot_tiles_cache: BTreeMap<(EntryID, Vec<TileID>), Vec<SlotTile>>,
    fetch_slot_meta_tile_cache: BTreeMap<(EntryID, TileID), SlotMetaTile>,
    fetch_slot_meta_tiles_cache: BTreeMap<(EntryID, Vec<TileID>), Vec<SlotMetaTile>>,
}

impl HTTPQueueDataSource {
    pub fn new(
        host: String,
        port: u16,
        queue: Arc<Mutex<Vec<Work>>>,
        initializer: Initializer,
    ) -> Self {
        log("INIT HTTPQueueDataSource");
        Self {
            host,
            port,
            client: reqwest::ClientBuilder::new()
                // .timeout(std::time::Duration::from_secs(5))
                // .gzip(true)
                // .brotli(true)
                .build()
                .unwrap(),
            // queue: Arc::new(Mutex::new(Vec::new())),
            queue,
            fetch_info: initializer.clone().entry_info,
            initializer: initializer.clone(),
            interval: Interval::default(),
            request_tiles_cache: BTreeMap::new(),
            fetch_slot_meta_tile_cache: BTreeMap::new(),
            fetch_slot_tile_cache: BTreeMap::new(),
            fetch_summary_tile_cache: BTreeMap::new(),
            fetch_summary_tiles_cache: BTreeMap::new(),
            fetch_slot_meta_tiles_cache: BTreeMap::new(),
            fetch_slot_tiles_cache: BTreeMap::new(),
        }
    }

    // empty queue and add results to respective caches
    fn process_queue(&mut self) {
        log("process_queue");
        let mut q = self.queue.lock().unwrap();

        for work in q.iter() {
            match work.process_type {
                ProcessType::FETCH_SLOT_META_TILE => {
                    // deserialize work.data into SlotMetaTile
                    let smt = serde_json::from_str::<SlotMetaTile>(&work.data).unwrap();
                    // add to cache
                    self.fetch_slot_meta_tile_cache
                        .insert((work.entry_id.clone(), smt.tile_id), smt);
                }
                ProcessType::FETCH_SLOT_TILE => {
                    // deserialize work.data into SlotTile
                    let st = serde_json::from_str::<SlotTile>(&work.data).unwrap();
                    // add to cache
                    self.fetch_slot_tile_cache
                        .insert((work.entry_id.clone(), st.tile_id), st);
                }

                ProcessType::REQUEST_TILES => {
                    // deserialize work.data into Vec<TileID>
                    let tiles = serde_json::from_str::<Vec<TileID>>(&work.data).unwrap();
                    // add to cache
                    self.request_tiles_cache
                        .insert((work.entry_id.clone(), work.interval.unwrap()), tiles);
                }
                ProcessType::FETCH_SUMMARY_TILE => {
                    // deserialize work.data into SummaryTile
                    let st = serde_json::from_str::<SummaryTile>(&work.data).unwrap();
                    // add to cache
                    self.fetch_summary_tile_cache
                        .insert((work.entry_id.clone(), st.tile_id), st);
                }
                ProcessType::FETCH_SLOT_META_TILES => {
                    // deserialize work.data into Vec<SlotMetaTile>
                    let smts = serde_json::from_str::<Vec<SlotMetaTile>>(&work.data).unwrap();
                    // add to cache
                    self.fetch_slot_meta_tiles_cache.insert(
                        (work.entry_id.clone(), work.tile_ids.clone().unwrap()),
                        smts,
                    );
                }
                ProcessType::FETCH_SLOT_TILES => {
                    // deserialize work.data into Vec<SlotTile>
                    let sts = serde_json::from_str::<Vec<SlotTile>>(&work.data).unwrap();
                    // add to cache
                    self.fetch_slot_tiles_cache
                        .insert((work.entry_id.clone(), work.tile_ids.clone().unwrap()), sts);
                }
                ProcessType::FETCH_SUMMARY_TILES => {
                    // deserialize work.data into Vec<SummaryTile>
                    let sts = serde_json::from_str::<Vec<SummaryTile>>(&work.data).unwrap();
                    // add to cache
                    console_log!("adding to cache: {:?}", work);
                    self.fetch_summary_tiles_cache
                        .insert((work.entry_id.clone(), work.tile_ids.clone().unwrap()), sts);
                }
                ProcessType::INTERVAL => {
                    // deserialize work.data into Interval
                    let interval = serde_json::from_str::<Interval>(&work.data).unwrap();
                    // add to cache
                    self.interval = interval;

                    // clear all the caches
                    self.request_tiles_cache.clear();
                    self.fetch_slot_meta_tile_cache.clear();
                    self.fetch_slot_tile_cache.clear();
                    self.fetch_summary_tile_cache.clear();
                    self.fetch_summary_tiles_cache.clear();
                    self.fetch_slot_meta_tiles_cache.clear();
                    self.fetch_slot_tiles_cache.clear();
                }
            }
        }
        // empty queue
        q.clear(); // ?
    }

    fn queue_work(&mut self, work: Work) {
        log("queue_work");
        let _work = work.clone();
        let url = match work.process_type {
            ProcessType::FETCH_SLOT_META_TILE => {
                format!("http://{}:{}/slot_meta_tile", self.host, self.port)
            }
            ProcessType::FETCH_SLOT_TILE => format!("http://{}:{}/slot_tile", self.host, self.port),
            ProcessType::REQUEST_TILES => format!("http://{}:{}/tiles", self.host, self.port),
            ProcessType::FETCH_SUMMARY_TILE => {
                format!("http://{}:{}/summary", self.host, self.port)
            }
            ProcessType::FETCH_SLOT_META_TILES => {
                format!("http://{}:{}/slot_meta_tiles", self.host, self.port)
            }
            ProcessType::FETCH_SLOT_TILES => {
                format!("http://{}:{}/slot_tiles", self.host, self.port)
            }
            ProcessType::FETCH_SUMMARY_TILES => {
                format!("http://{}:{}/summaries", self.host, self.port)
            }
            ProcessType::INTERVAL => format!("http://{}:{}/interval", self.host, self.port),
        };

        let body = match work.process_type {
            ProcessType::FETCH_SLOT_META_TILE => serde_json::to_string(&FetchRequest {
                entry_id: work.entry_id.clone(),
                tile_id: work.tile_id.unwrap(),
            })
            .unwrap(),
            ProcessType::FETCH_SLOT_TILE => serde_json::to_string(&FetchRequest {
                entry_id: work.entry_id.clone(),
                tile_id: work.tile_id.unwrap(),
            })
            .unwrap(),
            ProcessType::REQUEST_TILES => serde_json::to_string(&FetchTilesRequest {
                entry_id: work.entry_id.clone(),
                interval: work.interval.unwrap(),
            })
            .unwrap(),
            ProcessType::FETCH_SUMMARY_TILE => serde_json::to_string(&FetchRequest {
                entry_id: work.entry_id.clone(),
                tile_id: work.tile_id.unwrap(),
            })
            .unwrap(),
            ProcessType::FETCH_SLOT_META_TILES => serde_json::to_string(&FetchMultipleRequest {
                entry_id: work.entry_id.clone(),
                tile_ids: work.tile_ids.unwrap(),
            })
            .unwrap(),
            ProcessType::FETCH_SLOT_TILES => serde_json::to_string(&FetchMultipleRequest {
                entry_id: work.entry_id.clone(),
                tile_ids: work.tile_ids.unwrap(),
            })
            .unwrap(),
            ProcessType::FETCH_SUMMARY_TILES => serde_json::to_string(&FetchMultipleRequest {
                entry_id: work.entry_id.clone(),
                tile_ids: work.tile_ids.unwrap(),
            })
            .unwrap(),
            ProcessType::INTERVAL => "".to_string(),
        };

        let request = Request {
            method: "POST".to_owned(),
            url: url.to_string(),
            body: body.into(),
            headers: headers(&[("Accept", "*/*"), ("Content-Type", "javascript/json;")]),
        };
        // request.body = body.into();

        log("YEEHAW");
        log(&url.clone());
        let queue = self.queue.clone();
        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            // deserialize response into a vector of TileIDs

            let work = Work {
                entry_id: work.entry_id.clone(),
                tile_id: work.tile_id,
                tile_ids: _work.tile_ids.clone(),
                interval: work.interval,
                data: result.unwrap().text().unwrap().to_string(),
                process_type: work.process_type,
            };

            console_log!("ASYNC: pushing new work to queue: {:?}", work);
            queue.lock().unwrap().push(work);
        });
    }
}

impl DataSource for HTTPQueueDataSource {
    fn interval(&mut self) -> Interval {
        self.process_queue();
        console_log!("TESTINGTESTING");
        let work = Work {
            entry_id: EntryID::root(),
            tile_id: None,
            tile_ids: None,
            interval: None,
            data: "".to_string(),
            process_type: ProcessType::INTERVAL,
        };
        self.queue_work(work);
        self.interval
    }
    fn fetch_info(&mut self) -> EntryInfo {
        self.process_queue();

        self.fetch_info.clone()
    }

    fn init(&mut self) -> crate::data::Initializer {
        self.initializer.clone()
    }
    fn request_tiles(
        &mut self,
        entry_id: &EntryID,
        request_interval: Interval,
    ) -> Option<Vec<TileID>> {
        self.process_queue();

        // check cache for  entry_id, request_interval
        if let Some(tiles) = self
            .request_tiles_cache
            .get(&(entry_id.clone(), request_interval))
        {
            return Some(tiles.clone());
        } else {
            // queue work
            let work = Work {
                entry_id: entry_id.clone(),
                tile_id: None,
                tile_ids: None,
                interval: Some(request_interval),
                data: "".to_string(),
                process_type: ProcessType::REQUEST_TILES,
            };
            self.queue_work(work);
            return None;
        }
    }

    fn fetch_summary_tile(&mut self, entry_id: &EntryID, tile_id: TileID) -> Option<SummaryTile> {
        // check cache
        if let Some(st) = self
            .fetch_summary_tile_cache
            .get(&(entry_id.clone(), tile_id))
        {
            return Some(st.clone());
        } else {
            // queue work
            let work = Work {
                entry_id: entry_id.clone(),
                tile_id: Some(tile_id),
                interval: None,
                tile_ids: None,
                data: "".to_string(),
                process_type: ProcessType::FETCH_SUMMARY_TILE,
            };
            self.queue_work(work);
            return None;
        }
    }

    fn fetch_summary_tiles(
        &mut self,
        entry_id: &EntryID,
        tile_ids: Vec<TileID>,
    ) -> Option<Vec<SummaryTile>> {
        console_log!("fetch_summary_tiles with entry id");
        console_log!("{:?}", entry_id);
        // check cache
        if let Some(st) = self
            .fetch_summary_tiles_cache
            .get(&(entry_id.clone(), tile_ids.clone()))
        {
            return Some(st.clone());
        } else {
            // queue work
            let work = Work {
                entry_id: entry_id.clone(),
                tile_id: None,
                tile_ids: Some(tile_ids),
                interval: None,
                data: "".to_string(),
                process_type: ProcessType::FETCH_SUMMARY_TILES,
            };
            console_log!("adding summary work to queue {:?}", work);
            self.queue_work(work);
            return None;
        }
    }
    fn fetch_slot_tile(&mut self, entry_id: &EntryID, tile_id: TileID) -> Option<SlotTile> {
        self.process_queue();

        // check cache
        if let Some(st) = self.fetch_slot_tile_cache.get(&(entry_id.clone(), tile_id)) {
            return Some(st.clone());
        } else {
            // queue work
            let work = Work {
                entry_id: entry_id.clone(),
                tile_id: Some(tile_id),
                interval: None,
                tile_ids: None,
                data: "".to_string(),
                process_type: ProcessType::FETCH_SLOT_TILE,
            };
            self.queue_work(work);
            return None;
        }
    }

    fn fetch_slot_tiles(
        &mut self,
        entry_id: &EntryID,
        tile_ids: Vec<TileID>,
    ) -> Option<Vec<SlotTile>> {
        self.process_queue();

        // check cache
        if let Some(st) = self
            .fetch_slot_tiles_cache
            .get(&(entry_id.clone(), tile_ids.clone()))
        {
            return Some(st.clone());
        } else {
            // queue work
            let work = Work {
                entry_id: entry_id.clone(),
                tile_id: None,
                tile_ids: Some(tile_ids.clone()),
                interval: None,
                data: "".to_string(),
                process_type: ProcessType::FETCH_SLOT_TILES,
            };
            self.queue_work(work);
            return None;
        }
    }

    fn fetch_slot_meta_tile(
        &mut self,
        entry_id: &EntryID,
        tile_id: TileID,
    ) -> Option<SlotMetaTile> {
        self.process_queue();
        // check cache
        if let Some(smt) = self
            .fetch_slot_meta_tile_cache
            .get(&(entry_id.clone(), tile_id))
        {
            return Some(smt.clone());
        } else {
            // queue work
            let work = Work {
                entry_id: entry_id.clone(),
                tile_id: Some(tile_id),
                tile_ids: None,
                interval: None,
                data: "".to_string(),
                process_type: ProcessType::FETCH_SLOT_META_TILE,
            };
            self.queue_work(work);
            return None;
        }
    }
    fn fetch_slot_meta_tiles(
        &mut self,
        entry_id: &EntryID,
        tile_ids: Vec<TileID>,
    ) -> Option<Vec<SlotMetaTile>> {
        self.process_queue();
        // check cache
        if let Some(smt) = self
            .fetch_slot_meta_tiles_cache
            .get(&(entry_id.clone(), tile_ids.clone()))
        {
            return Some(smt.clone());
        } else {
            // queue work
            let work = Work {
                entry_id: entry_id.clone(),
                tile_id: None,
                tile_ids: Some(tile_ids.clone()),
                interval: None,
                data: "".to_string(),
                process_type: ProcessType::FETCH_SLOT_META_TILES,
            };
            self.queue_work(work);
            return None;
        }
    }
}
