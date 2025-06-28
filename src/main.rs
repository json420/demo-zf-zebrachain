use core::ops::Range;
use reqwest::header::{HeaderValue, RANGE};
use zf_zebrachain::{BLOCK, ChainStore, Hash};

const CHAIN_HASH: &[u8] = b"EWKP7KIM6RAB748D6VLT68BES58BHDOAQP8BS4FLBGPQ69ROF5HTDEHPPXWCGFCC";
const TAIL_BLOCK_HASH: &[u8] = b"DEFCQZHUV67IN5AUP7SYIHCR8NZNRZ7STS5H6RVM4WWSGMUZSAUKTXDQWD7JXE6C";

fn range_value(range: Range<u64>) -> HeaderValue {
    let value = format!("bytes={}-{}", range.start, range.end - 1);
    HeaderValue::from_str(&value).unwrap()
}

fn block_range(block_index: u64) -> HeaderValue {
    range_value(block_index * BLOCK as u64..(block_index + 1) * BLOCK as u64)
}

fn block_bulk_range(block_index: u64, count: u64) -> HeaderValue {
    assert!(count > 0);
    range_value(block_index * BLOCK as u64..(block_index + count) * BLOCK as u64)
}

fn main() {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let store = ChainStore::new(tmpdir.path());
    let chain_hash = Hash::from_z32(CHAIN_HASH).unwrap();

    let client = reqwest::blocking::Client::new();

    let url = format!("https://json420.github.io/chains/{}", chain_hash);

    println!("Downloading chain from {url}");
    let response = client
        .get(&url)
        .header(RANGE, block_bulk_range(0, 410))
        .send()
        .unwrap();

    println!("status: {}", response.status());
    for (key, value) in response.headers() {
        println!("{key}: {value:?}");
    }

    if response.status() != 206 {
        panic!("Expected 206");
    }
    let body = response.bytes().unwrap();
    let mut chain = store
        .create_chain(&body.slice(0..BLOCK), &chain_hash)
        .unwrap();
    for i in 1..body.len() / BLOCK {
        chain
            .append(&body.slice(i * BLOCK..(i + 1) * BLOCK))
            .unwrap();
    }

    loop {
        println!("");
        let response = client
            .get(&url)
            .header(RANGE, block_range(chain.count()))
            .send()
            .unwrap();
        println!("status: {}", response.status());
        for (key, value) in response.headers() {
            println!("{key}: {value:?}");
        }
        if response.status() != 206 {
            break;
        }
        let body = response.bytes().unwrap();
        if body.len() < BLOCK {
            break;
        }
        let block_hash = Hash::from_slice(&body.slice(0..40)).unwrap();
        println!("{block_hash}");
        chain.append(&body).unwrap();
    }
    assert_eq!(
        chain.tail().block_hash,
        Hash::from_z32(TAIL_BLOCK_HASH).unwrap()
    );
    assert_eq!(chain.count(), 420);
    println!("chain synced okay");
}
