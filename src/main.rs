use core::ops::Range;
use reqwest::header::{HeaderValue, RANGE};
use std::path::Path;
use zf_zebrachain::{BLOCK, Chain, ChainStore, Hash};

const CHAIN_HASH: &[u8] = b"K4UDPUCXUUWROI4T7X8DVKO7PXN8DFOUGA5COIRQZKW6BQGDW8MAIMACKJU69JHN5TMUXZLT";
const TAIL_BLOCK_HASH: &[u8] = b"MEJYM47KQ6F4ZSQ588YAP7DKPI8MNNGVQ45KW5XNM7UG8XAF6OJ7VAAP9TYARW8SXARIOS4J";

fn range_value(range: Range<u64>) -> HeaderValue {
    let value = format!("bytes={}-{}", range.start, range.end - 1);
    HeaderValue::from_str(&value).unwrap()
}

fn block_range(block_index: u64) -> HeaderValue {
    range_value(block_index * BLOCK as u64..(block_index + 1) * BLOCK as u64)
}

fn tail_range(count: u64) -> HeaderValue {
    let start = count * BLOCK as u64;
    let value = format!("bytes={start}-");
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

    fn get_range(&self, chain_hash: &Hash, range: HeaderValue) -> reqwest::blocking::Response {
        let url = format!("https://json420.github.io/chains/{}", chain_hash);
        println!("GET {url} {range:?}");
        let response = self.client.get(&url).header(RANGE, range).send().unwrap();
        println!("{}", response.status());
        for (key, value) in response.headers() {
            println!("    {key}: {value:?}");
        }
        response
    }

    fn init_chain(&self, chain_hash: &Hash) -> std::io::Result<Chain> {
        let response = self.get_range(chain_hash, block_range(0));
        let body = response.bytes().unwrap();
        let chain = self.store.create_chain(&body, chain_hash)?;
        println!("Created chain {chain_hash}");
        println!("");
        Ok(chain)
    }

    fn sync_chain(&self, chain_hash: &Hash) -> std::io::Result<Chain> {
        let mut chain = self.store.open_chain(chain_hash)?;
        let response = self.get_range(chain_hash, tail_range(chain.count()));
        if response.status() == 206 {
            let body = response.bytes().unwrap();
            for i in 0..body.len() / BLOCK {
                let buf = body.slice(i * BLOCK..(i + 1) * BLOCK);
                chain.append(&buf)?;
            }
            println!("Appended {} new blocks to {chain_hash}", body.len() / BLOCK);
        } else {
            assert_eq!(response.status(), 416);
            println!("No new blocks available for {chain_hash}");
        }
        println!("");
        Ok(chain)
    }
}

fn main() {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let downloader = Downloader::new(tmpdir.path());
    let chain_hash = Hash::from_z32(CHAIN_HASH).unwrap();

    let _chain = downloader.init_chain(&chain_hash).unwrap();
    let _chain = downloader.sync_chain(&chain_hash).unwrap();
    let chain = downloader.sync_chain(&chain_hash).unwrap();

    assert_eq!(
        chain.tail().block_hash,
        Hash::from_z32(TAIL_BLOCK_HASH).unwrap()
    );
    assert_eq!(chain.count(), 420);
    println!("Demo finished");
}
