use serde_bencode::de;
use std::env;
use std::fs;
use trident::metainfo::*;
use trident::peers::*;
use trident::tracker_communication::TrackerRequest;

fn main() {
    let args: Vec<String> = env::args().collect();
    let contents = fs::read(&args[1]).unwrap();
    let mi: Torrent = de::from_bytes::<Torrent>(&contents).unwrap();
    let peer_id: &str = "-TD1000-111111111111";
    let tr = TrackerRequest::start_request(&mi, String::from(peer_id), 6881);
    // println!("{}", tr.to_url(&mi.announce.unwrap()));

    let response = tr.send_request(&mi);
    println!("{:?}", response);
    if response.is_err() {
        return;
    }
    let response = response.unwrap();

    if response.peers.len() == 0 {
        return;
    }
    let peer_addr = format!("{}:{}", response.peers[0].ip, response.peers[0].port);
    let mut peer = Peer::new(peer_addr, mi.info.clone()).unwrap();
    match peer.handshake(&mi.info_hash(), peer_id) {
        Ok(response) => {
            println!("Response: {:?}", response);
        }
        Err(e) => {
            println!("{}", e);
        }
    }

    match peer.next_message() {
        Ok(message) => {
            println!("Message: {:?}", message);
        }
        Err(e) => {
            println!("{}", e);
        }
    }

    match peer.next_message() {
        Ok(message) => {
            println!("Message: {:?}", message);
        }
        Err(e) => {
            println!("{}", e);
        }
    }

    // match peer.get_bitfield() {
    //     Ok(response) => {
    //         println!("Response: {:?}", response)
    //     },
    //     Err(e) => {
    //         println!("{}", e)
    //     }
    // }
}
