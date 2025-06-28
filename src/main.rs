use zf_zebrachain::{BLOCK, ChainStore, Hash};

const CHAIN_HASH: &str = "EWKP7KIM6RAB748D6VLT68BES58BHDOAQP8BS4FLBGPQ69ROF5HTDEHPPXWCGFCC";
const TAIL_BLOCK_HASH: &[u8] = b"DEFCQZHUV67IN5AUP7SYIHCR8NZNRZ7STS5H6RVM4WWSGMUZSAUKTXDQWD7JXE6C";

fn main() {
    let tmpdir = tempfile::TempDir::new().unwrap();
    let store = ChainStore::new(tmpdir.path());
    let chain_hash = Hash::from_z32(CHAIN_HASH.as_bytes()).unwrap();

    let url = format!("https://json420.github.io/chains/{}", CHAIN_HASH);
    let response = reqwest::blocking::get(url).unwrap();

    if response.status() == 200 {
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
    }
}
