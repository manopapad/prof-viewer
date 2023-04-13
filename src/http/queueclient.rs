use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    data::{DataSource, EntryID, EntryInfo, SlotMetaTile, SlotTile, SummaryTile, TileID},
    queue::queue::{ProcessType, Work, WorkQueue},
    timestamp::Interval,
};
use ehttp;

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

pub struct HTTPQueueDataSource {
    pub host: String,
    pub port: u16,
    pub client: reqwest::Client,
    pub queue: WorkQueue,
    pub fetch_info: EntryInfo,
    pub interval: Interval,
    request_tiles_cache: BTreeMap<(EntryID, Interval), Vec<TileID>>,
    fetch_summary_tile_cache: BTreeMap<(EntryID, TileID), SummaryTile>,
    fetch_slot_tile_cache: BTreeMap<(EntryID, TileID), SlotTile>,
    fetch_slot_meta_tile_cache: BTreeMap<(EntryID, TileID), SlotMetaTile>,
}

impl HTTPQueueDataSource {
    pub fn new(host: String, port: u16, queue: WorkQueue, fetch_info: EntryInfo) -> Self {
        Self {
            host,
            port,
            client: reqwest::ClientBuilder::new()
                .timeout(std::time::Duration::from_secs(5))
                .gzip(true)
                .brotli(true)
                .build()
                .unwrap(),
            queue,
            fetch_info,
            interval: Interval::default(),
            request_tiles_cache: BTreeMap::new(),
            fetch_slot_meta_tile_cache: BTreeMap::new(),
            fetch_slot_tile_cache: BTreeMap::new(),
            fetch_summary_tile_cache: BTreeMap::new(),
        }
    }

    // empty queue and add results to respective caches
    fn process_queue(&mut self) {
        let q = self.queue.lock().unwrap();

        for work in q.iter() {
            match work.process_type {
                ProcessType::FETCH_SLOT_META_TILE => {
                    // deserialize work.data into SlotMetaTile
                    let smt = serde_json::from_str::<SlotMetaTile>(&work.data).unwrap();
                    // add to cache
                    self.fetch_slot_meta_tile_cache
                        .insert((work.entry_id, smt.tile_id), smt);
                }
                ProcessType::FETCH_SLOT_TILE => {
                    // deserialize work.data into SlotTile
                    let st = serde_json::from_str::<SlotTile>(&work.data).unwrap();
                    // add to cache
                    self.fetch_slot_tile_cache
                        .insert((work.entry_id, st.tile_id), st);
                }

                ProcessType::REQUEST_TILES => {
                    // deserialize work.data into Vec<TileID>
                    let tiles = serde_json::from_str::<Vec<TileID>>(&work.data).unwrap();
                    // add to cache
                    self.request_tiles_cache
                        .insert((work.entry_id, work.interval.unwrap()), tiles);
                }
                ProcessType::FETCH_SUMMARY => {
                    // deserialize work.data into SummaryTile
                    let st = serde_json::from_str::<SummaryTile>(&work.data).unwrap();
                    // add to cache
                    self.fetch_summary_tile_cache
                        .insert((work.entry_id, st.tile_id), st);
                }
            }
        }
        // empty queue
        self.queue.lock().unwrap().clear(); // ?
    }

    fn queue_work(&mut self, work: Work) {
        let url = match work.process_type {
            ProcessType::FETCH_SLOT_META_TILE => {
                format!("http://{}:{}/slot_meta_tile", self.host, self.port)
            }
            ProcessType::FETCH_SLOT_TILE => format!("http://{}:{}/slot_tile", self.host, self.port),
            ProcessType::REQUEST_TILES => format!("http://{}:{}/tiles", self.host, self.port),
            ProcessType::FETCH_SUMMARY => format!("http://{}:{}/summary", self.host, self.port),
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
            ProcessType::FETCH_SUMMARY => serde_json::to_string(&FetchRequest {
                entry_id: work.entry_id.clone(),
                tile_id: work.tile_id.unwrap(),
            })
            .unwrap(),
        };

        let request = ehttp::Request::get(url);
        request.body = body.into();

        let queue = self.queue.clone();
        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            // deserialize response into a vector of TileIDs

            let work = Work {
                entry_id: work.entry_id.clone(),
                tile_id: work.tile_id,
                interval: work.interval,
                data: result.unwrap().text().unwrap().to_string(),
                process_type: work.process_type,
            };
            queue.lock().unwrap().push(work);
        });
    }
}

impl DataSource for HTTPQueueDataSource {
    fn interval(&mut self) -> Interval {
        self.process_queue();

        self.interval
    }
    fn fetch_info(&mut self) -> EntryInfo {
        self.process_queue();

        self.fetch_info.clone()
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
                data: "".to_string(),
                process_type: ProcessType::FETCH_SUMMARY,
            };
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
                data: "".to_string(),
                process_type: ProcessType::FETCH_SLOT_TILE,
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
                interval: None,
                data: "".to_string(),
                process_type: ProcessType::FETCH_SLOT_META_TILE,
            };
            self.queue_work(work);
            return None;
        }
    }
}
