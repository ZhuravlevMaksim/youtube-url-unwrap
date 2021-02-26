use youtube_url_unwrap::platform::Extractor;

#[tokio::main]
async fn  main() -> Result<(), Box<dyn std::error::Error>> {
    let unwrap = Extractor::new().get_opus_stream("ASArOYTKwW4").await?;
    println!("{:#?}", unwrap);
    Ok(())
}
