use youtube_url_unwrap::Extractor;

#[tokio::main]
async fn  main() -> Result<(), Box<dyn std::error::Error>> {
    let unwrap = Extractor::new().get_opus_stream("ASArOYTKwW4").await?;
    println!("{:#?}", unwrap.file_name());
    println!("{:#?}", unwrap);
    Ok(())
}
