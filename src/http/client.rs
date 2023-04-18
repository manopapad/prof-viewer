// use crate::{
//     data::{DataSource, EntryID, EntryInfo, SlotMetaTile, SlotTile, SummaryTile, TileID},
//     timestamp::Interval,
// };

// use super::schema::{FetchRequest, FetchTilesRequest};

// pub struct HTTPDataSource {
//     pub host: String,
//     pub port: u16,
//     pub client: reqwest::blocking::Client,
// }

// impl HTTPDataSource {
//     pub fn new(host: String, port: u16) -> Self {
//         Self {
//             host,
//             port,
//             client: reqwest::blocking::ClientBuilder::new()
//                 .timeout(std::time::Duration::from_secs(5))
//                 .gzip(true)
//                 .brotli(true)
//                 .build()
//                 .unwrap(),
//         }
//     }
// }

// impl DataSource for HTTPDataSource {
//     fn interval(&mut self) -> Interval {
//         let resp = self
//             .client
//             .get(format!("http://{}:{}/interval", self.host, self.port))
//             .send();
//         resp.unwrap().json::<Interval>().unwrap()
//     }
//     fn fetch_info(&mut self) -> EntryInfo {
//         let resp = self
//             .client
//             .get(format!("http://{}:{}/info", self.host, self.port))
//             .send();
//         resp.unwrap().json::<EntryInfo>().unwrap()
//     }
//     fn request_tiles(&mut self, entry_id: &EntryID, request_interval: Interval) -> Vec<TileID> {
//         let resp = self
//             .client
//             .get(format!("http://{}:{}/tiles", self.host, self.port))
//             .json(&FetchTilesRequest {
//                 entry_id: entry_id.clone(),
//                 interval: request_interval,
//             })
//             .send();
//         resp.unwrap().json::<Vec<TileID>>().unwrap()
//     }
//     fn fetch_summary_tile(&mut self, entry_id: &EntryID, tile_id: TileID) -> SummaryTile {
//         let resp = self
//             .client
//             .get(format!("http://{}:{}/summary_tile", self.host, self.port))
//             .json(&FetchRequest {
//                 entry_id: entry_id.clone(),
//                 tile_id,
//             })
//             .send();
//         resp.unwrap().json::<SummaryTile>().unwrap()
//     }
//     fn fetch_slot_tile(&mut self, entry_id: &EntryID, tile_id: TileID) -> SlotTile {
//         let resp = self
//             .client
//             .get(format!("http://{}:{}/slot_tile", self.host, self.port))
//             .json(&FetchRequest {
//                 entry_id: entry_id.clone(),
//                 tile_id,
//             })
//             .send();
//         resp.unwrap().json::<SlotTile>().unwrap()
//     }
//     fn fetch_slot_meta_tile(&mut self, entry_id: &EntryID, tile_id: TileID) -> SlotMetaTile {
//         let resp = self
//             .client
//             .get(format!("http://{}:{}/slot_meta_tile", self.host, self.port))
//             .json(&FetchRequest {
//                 entry_id: entry_id.clone(),
//                 tile_id,
//             })
//             .send();
//         resp.unwrap().json::<SlotMetaTile>().unwrap()
//     }
// }
