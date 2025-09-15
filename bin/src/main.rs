#[tokio::main]
async fn main() -> gm_tui::Result<()> {
    // TODO improve argument parsing
    let args: Vec<String> = std::env::args().skip(1).collect();

    gm_tui::run(args).await?;

    Ok(())
}
