// use std::sync::{Arc, Mutex};

use crate::{
    data::{EntryID, Initializer, SlotMetaTile, SlotTile, SummaryTile, TileID},
    timestamp::Interval,
};

// make a template struct called Data
#[derive(Clone, Debug, PartialEq)]
pub enum Data<T> {
    Requested,
    Ready(T),
}

#[derive(Clone, Debug)]
pub enum Work {
    FetchInfo(Data<Initializer>),
    FetchSlotMetaTile(EntryID, TileID, Data<SlotMetaTile>),
    FetchSlotTile(EntryID, TileID, Data<SlotTile>),
    FetchTiles(EntryID, Interval, Data<Vec<TileID>>),
    FetchSummaryTile(EntryID, TileID, Data<SummaryTile>),
    FetchInterval(Data<Interval>),
}
