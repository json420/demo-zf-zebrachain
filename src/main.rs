use core::ops::Range;
use reqwest::header::{HeaderValue, RANGE};
use zf_zebrachain::{BLOCK, ChainStore, Hash};

const CHAIN_HASH: &[u8] = b"EWKP7KIM6RAB748D6VLT68BES58BHDOAQP8BS4FLBGPQ69ROF5HTDEHPPXWCGFCC";
const TAIL_BLOCK_HASH: &[u8] = b"DEFCQZHUV67IN5AUP7SYIHCR8NZNRZ7STS5H6RVM4WWSGMUZSAUKTXDQWD7JXE6C";

fn range_header(range: Range<usize>) -> HeaderValue {
    let value = format!("bytes={}-{}", range.start, range.end - 1);
    HeaderValue::from_str(&value).unwrap()
}

fn main() {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let store = ChainStore::new(tmpdir.path());
    let chain_hash = Hash::from_z32(CHAIN_HASH).unwrap();

    let client = reqwest::blocking::Client::new();

    let url = format!("https://json420.github.io/chains/{}", chain_hash);
    println!("Downloading chain from {url}");
    let response = client
        .get(url)
        .header(RANGE, range_header(0..BLOCK))
        .send()
        .unwrap();

    println!("status: {}", response.status());
    for (key, value) in response.headers() {
        println!("{key}: {value:?}");
    }

    if response.status() == 206 {
        let body = response.bytes().unwrap();
        let mut chain = store
            .create_chain(&body.slice(0..BLOCK), &chain_hash)
            .unwrap();
        for index in 1..body.len() / BLOCK {
            chain
                .append(&body.slice(index * BLOCK..(index + 1) * BLOCK))
                .unwrap();
        }
        assert_eq!(
            chain.tail().block_hash,
            Hash::from_z32(TAIL_BLOCK_HASH).unwrap()
        );
        assert_eq!(chain.count(), 420);
    }
}
