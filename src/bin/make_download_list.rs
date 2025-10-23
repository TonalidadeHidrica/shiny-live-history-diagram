use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use shiny_live_history_diagram::WikiFetcher;

#[derive(Parser)]
struct Opts {
    #[arg(long, default_value = "./ignore/list.json")]
    output_path: PathBuf,
}

fn main() -> Result<()> {
    env_logger::builder().format_timestamp_nanos().init();
    let opts = Opts::parse();

    let mut fetcher = WikiFetcher::default();
    let result = fetcher.fetch("全曲一覧".into())?;

    println!("{}", result.html());

    Ok(())
}
