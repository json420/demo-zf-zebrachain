use zf_zebrachain::{BLOCK, ChainStore, Hash};

const CHAIN_HASH: &str = "EWKP7KIM6RAB748D6VLT68BES58BHDOAQP8BS4FLBGPQ69ROF5HTDEHPPXWCGFCC";

fn main() {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let store = ChainStore::new(tmpdir.path());
    let chain_hash = Hash::from_z32(CHAIN_HASH.as_bytes()).unwrap();

    let url = format!("https://json420.github.io/chains/{}", CHAIN_HASH);
    let response = reqwest::blocking::get(url).unwrap();

    if response.status() == 200 {
        let body = response.bytes().unwrap();
        let mut chain = store.create_chain(&body.slice(0..BLOCK), &chain_hash).unwrap();
        println!("OK {}", body.len());
    }
}
