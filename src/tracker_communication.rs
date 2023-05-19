use crate::metainfo::Torrent;
use reqwest;
use serde_bencode::de;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Event {
    Started,
    Stopped,
    Completed,
}

#[derive(Debug)]
pub struct TrackerRequest {
    pub info_hash: String,
    pub peer_id: String,
    pub port: i64,
    pub uploaded: i64,
    pub downloaded: i64,
    pub left: i64,
    pub compact: i32,
    pub no_peer_id: i32,
    pub event: Event,
    pub numwant: i32,
    pub trackerid: Option<String>,
}

impl TrackerRequest {
    pub fn start_request(torrent: &Torrent, peer_id: String, port: i64) -> Self {
        TrackerRequest {
            info_hash: torrent.info_hash_urlencoded(),
            peer_id,
            port,
            uploaded: 0,
            downloaded: 0,
            left: 0,
            compact: 1,
            no_peer_id: 1,
            event: Event::Started,
            numwant: 50,
            trackerid: None,
        }
    }

    pub fn to_url(&self, announce: &str) -> String {
        let event_string = match self.event {
            Event::Started => "started",
            Event::Stopped => "stopped",
            Event::Completed => "completed",
        };

        let tracker_id_field = match &self.trackerid {
            None => String::new(),
            Some(tracker_id) => format!("&trackerid={}", tracker_id),
        };

        format!(
            "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}&no_peer_id={}&event={}&numwant={}{}",
            announce,
            self.info_hash,
            self.peer_id,
            self.port,
            self.uploaded,
            self.downloaded,
            self.left,
            self.compact,
            self.no_peer_id,
            event_string,
            self.numwant,
            tracker_id_field
        )
    }

    pub fn send_request(&self, torrent: &Torrent) -> Result<TrackerResponse, reqwest::Error> {
        let url = match &torrent.announce {
            None => {
                panic!("DHT is not supported.");
            }
            Some(announce) => self.to_url(announce),
        };

        let response_body = reqwest::blocking::get(url)?.bytes()?;
        println!("Response Body: {:?}", response_body);
        Ok(de::from_bytes::<TrackerResponse>(&response_body).unwrap())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PeerInfo {
    #[serde(default)]
    #[serde(rename = "peer id")]
    pub peer_id: Option<String>,
    pub ip: String,
    pub port: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerResponse {
    #[serde(default)]
    #[serde(rename = "failure reason")]
    pub failure_reason: Option<String>,
    #[serde(default)]
    #[serde(rename = "warning message")]
    pub warning_message: Option<String>,
    #[serde(default)]
    pub interval: Option<i64>,
    #[serde(default)]
    #[serde(rename = "min interval")]
    pub min_interval: Option<i64>,
    #[serde(default)]
    #[serde(rename = "tracker id")]
    pub tracker_id: Option<String>,
    #[serde(default)]
    pub complete: Option<i64>,
    #[serde(default)]
    pub incomplete: Option<i64>,
    #[serde(default)]
    pub peers: Vec<PeerInfo>,
}
