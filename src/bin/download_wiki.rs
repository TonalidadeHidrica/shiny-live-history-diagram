use std::{collections::HashSet, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use fs_err::{File, create_dir_all};
use log::info;
use shiny_live_history_diagram::{
    WikiFetcher,
    song_list::{self, GenreCode},
};

#[derive(Parser)]
struct Opts {
    genres: Vec<GenreCode>,

    #[arg(long, default_value = "./ignore/list.json")]
    list_json_path: PathBuf,
    #[arg(long, default_value = "./ignore/wiki_html/")]
    output_dir_path: PathBuf,
}

fn main() -> Result<()> {
    env_logger::builder().format_timestamp_nanos().init();
    let opts = Opts::parse();

    let list: song_list::List = serde_json::from_reader(File::open(opts.list_json_path)?)?;
    let mut fetcher = WikiFetcher::default();

    let genres = HashSet::<_>::from_iter(&opts.genres);
    create_dir_all(&opts.output_dir_path)?;
    for song in &list.songs {
        if !genres.contains(&song.genre) {
            continue;
        }
        let path = opts.output_dir_path.join(song.link.to_file_name()?);
        if path.exists() {
            info!("Song {} exists; skipping.", song.title);
            continue;
        }
        fs_err::write(&path, fetcher.fetch(&song.link)?)?;
        info!("Saved song {} as {path:?}", song.title);
    }

    Ok(())
}
