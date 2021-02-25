use youtube_url_unwrap::unwrap_url;

fn main() {
    let unwrap = unwrap_url("https://httpbin.org/ip");
    println!("{:#?}", unwrap);
}