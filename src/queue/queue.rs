// use std::sync::{Arc, Mutex};


use crate::{
    data::{EntryID, TileID},
    timestamp::Interval,
};

#[derive(Clone, Debug)]
pub enum ProcessType {
    FETCH_SLOT_META_TILE,
    FETCH_SLOT_TILE,
    FETCH_TILES,
    FETCH_SUMMARY_TILE,
    INTERVAL,
}

#[derive(Clone, Debug)]
pub struct Work {
    pub entry_id: EntryID,
    pub tile_id: Option<TileID>,
    pub tile_ids: Option<Vec<TileID>>,
    pub interval: Option<Interval>,
    pub data: String,
    pub process_type: ProcessType,
}

// #[derive(Clone)]
// pub type WorkQueue = Arc<Mutex<Vec<Work>>>;
