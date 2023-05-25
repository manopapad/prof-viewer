// use std::sync::{Arc, Mutex};

use crate::{
    data::{EntryID, TileID},
    timestamp::Interval,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ProcessType {
    FetchInfo,
    FetchSlotMetaTile,
    FetchSlotTile,
    FetchTiles,
    FetchSummaryTile,
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
