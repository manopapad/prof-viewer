use std::collections::BTreeMap;

use crate::data::{DataSource, EntryID, Initializer, SlotMetaTile, SlotTile, SummaryTile, TileID};
use crate::timestamp::Interval;

pub trait DeferredDataSource {
    fn fetch_info(&mut self);
    fn get_info(&mut self) -> Option<Initializer>;
    fn fetch_tiles(&mut self, entry_id: EntryID, request_interval: Interval);
    fn get_tiles(&mut self, entry_id: EntryID) -> Vec<TileID>;
    fn fetch_summary_tile(&mut self, entry_id: EntryID, tile_id: TileID);
    fn get_summary_tiles(&mut self) -> Vec<SummaryTile>;
    fn fetch_slot_tile(&mut self, entry_id: EntryID, tile_id: TileID);
    fn get_slot_tile(&mut self) -> Vec<SlotTile>;
    fn fetch_slot_meta_tile(&mut self, entry_id: EntryID, tile_id: TileID);
    fn get_slot_meta_tile(&mut self) -> Vec<SlotMetaTile>;
}

pub struct DeferredDataSourceWrapper {
    data_source: Box<dyn DataSource>,
    tiles: BTreeMap<EntryID, Vec<TileID>>,
    summary_tiles: Vec<SummaryTile>,
    slot_tiles: Vec<SlotTile>,
    slot_meta_tiles: Vec<SlotMetaTile>,
}

impl DeferredDataSourceWrapper {
    pub fn new(data_source: Box<dyn DataSource>) -> Self {
        Self {
            data_source,
            summary_tiles: Vec::new(),
            slot_tiles: Vec::new(),
            slot_meta_tiles: Vec::new(),
            tiles: BTreeMap::new(),
        }
    }
}

impl DeferredDataSource for DeferredDataSourceWrapper {
    fn fetch_info(&mut self) {}

    fn get_info(&mut self) -> Option<Initializer> {
        Some(self.data_source.fetch_info())
    }

    fn fetch_tiles(&mut self, entry_id: EntryID, request_interval: Interval) {
        let tiles = self.data_source.request_tiles(&entry_id, request_interval);

        let value: Vec<TileID> = self
            .tiles
            .entry(entry_id.clone())
            .or_insert(tiles.clone())
            .clone();

        self.tiles
            .entry(entry_id)
            .or_insert(tiles.clone())
            .extend(tiles.into_iter().filter(|&x| !value.contains(&x)));
    }

    fn get_tiles(&mut self, entry_id: EntryID) -> Vec<TileID> {
        self.tiles.get(&entry_id).unwrap().clone()
    }

    fn fetch_summary_tile(&mut self, entry_id: EntryID, tile_id: TileID) {
        let sum_tile = self.data_source.fetch_summary_tile(&entry_id, tile_id);

        self.summary_tiles.push(sum_tile);
    }

    fn get_summary_tiles(&mut self) -> Vec<SummaryTile> {
        let ret = self.summary_tiles.clone();
        self.summary_tiles.clear();
        ret
    }
    fn fetch_slot_tile(&mut self, entry_id: EntryID, tile_id: TileID) {
        let slot_tile = self.data_source.fetch_slot_tile(&entry_id, tile_id);

        self.slot_tiles.push(slot_tile);
    }

    fn get_slot_tile(&mut self) -> Vec<SlotTile> {
        let ret = self.slot_tiles.clone();

        self.slot_tiles.clear();
        ret
    }

    fn fetch_slot_meta_tile(&mut self, entry_id: EntryID, tile_id: TileID) {
        let slot_meta_tile = self.data_source.fetch_slot_meta_tile(&entry_id, tile_id);

        self.slot_meta_tiles.push(slot_meta_tile);
    }

    fn get_slot_meta_tile(&mut self) -> Vec<SlotMetaTile> {
        let ret = self.slot_meta_tiles.clone();
        self.slot_meta_tiles.clear();
        ret
    }
}
