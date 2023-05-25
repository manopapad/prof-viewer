use serde::{Deserialize, Serialize};

use crate::{
    data::{EntryID, TileID},
    timestamp::Interval,
};

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