use std::sync::{Arc, Mutex};

use egui::Window;

use crate::{
    data::{EntryID, TileID},
    timestamp::Interval,
};

pub enum ProcessType {
    FETCH_SLOT_META_TILE,
    FETCH_SLOT_TILE,
    REQUEST_TILES,
    FETCH_SUMMARY,
}

pub struct Work {
    pub entry_id: EntryID,
    pub tile_id: Option<TileID>,
    pub interval: Option<Interval>,
    pub data: String,
    pub process_type: ProcessType,
}

// #[derive(Clone)]
pub type WorkQueue = Arc<Mutex<Vec<Work>>>;
