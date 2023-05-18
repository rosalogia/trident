use serde_bencode::de;
use std::fs;
use trident::metainfo::*;
use trident::tracker_communication::TrackerRequest;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let contents = fs::read(&args[1]).unwrap();
    let mi: Torrent = de::from_bytes::<Torrent>(&contents).unwrap();
    let tr = TrackerRequest::start_request(&mi, String::from("-TD1000-111111111111"), 6881);
    let response = tr.send_request(&mi);
    println!("{:?}", response);
}
