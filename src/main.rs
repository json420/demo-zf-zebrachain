use core::ops::Range;
use reqwest::header::{HeaderValue, RANGE};
use std::path::Path;
use zf_zebrachain::{BLOCK, Chain, ChainStore, Hash};

const CHAIN_HASH: &[u8] = b"EWKP7KIM6RAB748D6VLT68BES58BHDOAQP8BS4FLBGPQ69ROF5HTDEHPPXWCGFCC";
const TAIL_BLOCK_HASH: &[u8] = b"DEFCQZHUV67IN5AUP7SYIHCR8NZNRZ7STS5H6RVM4WWSGMUZSAUKTXDQWD7JXE6C";

fn range_value(range: Range<u64>) -> HeaderValue {
    let value = format!("bytes={}-{}", range.start, range.end - 1);
    println!("{value}");
    HeaderValue::from_str(&value).unwrap()
}

fn block_range(block_index: u64) -> HeaderValue {
    range_value(block_index * BLOCK as u64..(block_index + 1) * BLOCK as u64)
}

fn block_bulk_range(block_index: u64, count: u64) -> HeaderValue {
    assert!(count > 0);
    range_value(block_index * BLOCK as u64..(block_index + count) * BLOCK as u64)
}

fn tail_range(count: u64) -> HeaderValue {
    let start = count * BLOCK as u64;
    let value = format!("bytes={start}-");
    println!("{value}");
    HeaderValue::from_str(&value).unwrap()
}

struct Downloader {
    client: reqwest::blocking::Client,
    store: ChainStore,
}

impl Downloader {
    fn new(dir: &Path) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            store: ChainStore::new(dir),
        }
    }

    fn get(&self, chain_hash: &Hash) -> reqwest::blocking::RequestBuilder {
        let url = format!("https://json420.github.io/chains/{}", chain_hash);
        self.client.get(&url)
    }

    fn init_chain(&self, chain_hash: &Hash) -> std::io::Result<Chain> {
        let response = self
            .get(chain_hash)
            .header(RANGE, block_range(0))
            .send()
            .unwrap();
        println!("status: {}", response.status());
        for (key, value) in response.headers() {
            println!("{key}: {value:?}");
        }
        let body = response.bytes().unwrap();
        self.store.create_chain(&body, chain_hash)
    }

    fn sync_chain(&self, chain_hash: &Hash) -> std::io::Result<Chain> {
        let mut chain = self.store.open_chain(chain_hash)?;
        let response = self
            .get(chain_hash)
            .header(RANGE, tail_range(chain.count()))
            .send()
            .unwrap();
        println!("status: {}", response.status());
        for (key, value) in response.headers() {
            println!("{key}: {value:?}");
        }
        let body = response.bytes().unwrap();
        for i in 0..body.len() / BLOCK {
            let buf = body.slice(i * BLOCK..(i + 1) * BLOCK);
            chain.append(&buf)?;
        }
        Ok(chain)
    }
}

fn main() {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let downloader = Downloader::new(tmpdir.path());
    let chain_hash = Hash::from_z32(CHAIN_HASH).unwrap();

    let chain = downloader.init_chain(&chain_hash).unwrap();
    let chain = downloader.sync_chain(&chain_hash).unwrap();

    assert_eq!(
        chain.tail().block_hash,
        Hash::from_z32(TAIL_BLOCK_HASH).unwrap()
    );
    assert_eq!(chain.count(), 420);
    println!("chain synced okay");
}
