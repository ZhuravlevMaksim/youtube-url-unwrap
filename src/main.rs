use youtube_url_unwrap::{get_opus_stream};

fn main() {
    let unwrap = get_opus_stream("a0zUnqF5tNs");
    println!("{:#?}", unwrap);
}