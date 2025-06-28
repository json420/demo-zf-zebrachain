fn main() {
    let response = reqwest::blocking::get("https://json420.github.io/chains/EWKP7KIM6RAB748D6VLT68BES58BHDOAQP8BS4FLBGPQ69ROF5HTDEHPPXWCGFCC").unwrap();
    println!("done");
}
