use youtube_url_unwrap::platform::get_opus_stream;

#[tokio::main]
async fn  main() -> Result<(), Box<dyn std::error::Error>> {
    let unwrap = get_opus_stream("a0zUnqF5tNs").await?;
    println!("{:#?}", unwrap);
    Ok(())
}
