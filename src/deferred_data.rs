use crate::data::{DataSource, EntryID, EntryInfo, SlotMetaTile, SlotTile, SummaryTile, TileID};
use crate::timestamp::Interval;

pub trait DeferredDataSource {
    fn interval(&mut self) -> Interval;
    fn fetch_info(&mut self) -> &EntryInfo;
    fn request_tiles(&mut self, entry_id: &EntryID, request_interval: Interval) -> Vec<TileID>;
    fn fetch_summary_tile(&mut self, entry_id: &EntryID, tile_id: TileID);
    fn get_summary_tiles(&mut self) -> Vec<SummaryTile>;
    fn fetch_slot_tile(&mut self, entry_id: &EntryID, tile_id: TileID);
    fn get_slot_tile(&mut self) -> Vec<SlotTile>;
    fn fetch_slot_meta_tile(&mut self, entry_id: &EntryID, tile_id: TileID);
    fn get_slot_meta_tile(&mut self) -> Vec<SlotMetaTile>;
}

pub struct DeferredDataSourceWrapper {
    data_source: Box<dyn DataSource>,
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
        }
    }
}

impl DeferredDataSource for DeferredDataSourceWrapper {
    fn interval(&mut self) -> Interval {
        self.data_source.interval()
    }

    fn fetch_info(&mut self) -> &EntryInfo {
        self.data_source.fetch_info()
    }

    fn request_tiles(&mut self, entry_id: &EntryID, request_interval: Interval) -> Vec<TileID> {
        self.data_source.request_tiles(entry_id, request_interval)
    }

    fn fetch_summary_tile(&mut self, entry_id: &EntryID, tile_id: TileID) {
        self.summary_tiles
            .push(self.data_source.fetch_summary_tile(entry_id, tile_id));
    }

    fn get_summary_tiles(&mut self) -> Vec<SummaryTile> {
        std::mem::take(&mut self.summary_tiles)
    }

    fn fetch_slot_tile(&mut self, entry_id: &EntryID, tile_id: TileID) {
        self.slot_tiles
            .push(self.data_source.fetch_slot_tile(entry_id, tile_id));
    }

    fn get_slot_tile(&mut self) -> Vec<SlotTile> {
        std::mem::take(&mut self.slot_tiles)
    }

    fn fetch_slot_meta_tile(&mut self, entry_id: &EntryID, tile_id: TileID) {
        self.slot_meta_tiles
            .push(self.data_source.fetch_slot_meta_tile(entry_id, tile_id));
    }

    fn get_slot_meta_tile(&mut self) -> Vec<SlotMetaTile> {
        std::mem::take(&mut self.slot_meta_tiles)
    }
}
