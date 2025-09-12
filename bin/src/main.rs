#[tokio::main]
async fn main() -> gm_tui::Result<()> {
    preload_hook();

    // TODO improve argument parsing
    let args: Vec<String> = std::env::args().skip(1).collect();

    gm_tui::run(args).await?;

    Ok(())
}

fn preload_hook() {
    // TODO move this. its better to do it when it is needed instead of always
    gm_utils::network::NetworkStore::sort_config().expect("NetworkStore::sort_config() failed");
}
