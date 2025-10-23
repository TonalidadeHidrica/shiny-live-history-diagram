use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use clap::Parser;
use itertools::Itertools;
use log::{error, info};
use shiny_live_history_diagram::{WikiFetcher, selector, song_list};

#[derive(Parser)]
struct Opts {
    #[arg(long, default_value = "./ignore/list.json")]
    output_json_path: PathBuf,
}

fn main() -> Result<()> {
    env_logger::builder().format_timestamp_nanos().init();
    let opts = Opts::parse();

    let mut fetcher = WikiFetcher::default();
    let result = fetcher.fetch("全曲一覧".into())?;

    let mut res = song_list::List { songs: vec![] };
    for tr in result.select(selector!("div.content tbody > tr")) {
        if tr.select(selector!("th")).next().is_some() {
            // Skip header row
            continue;
        }
        match tr.select(selector!("td")).collect_vec()[..] {
            [td] => {
                // Skip sub-header row
                info!("Parsing {}", td.text().collect::<String>().trim());
            }
            [genre, title, artist, date, material] => {
                let genre = genre.text().collect::<String>().into();
                let (title, link) = {
                    let a = title
                        .select(selector!("a"))
                        .next()
                        .with_context(|| format!("`<a>` not found: {}", title.html()))?;
                    let title = a.text().collect::<String>().into();
                    let href = a
                        .attr("href")
                        .with_context(|| format!("Attribute `href` not found: {}", a.html()))?;
                    let link = href
                        .strip_prefix('/')
                        .with_context(|| format!("`href` does not start with `/`: {href:?}"))?
                        .to_owned()
                        .into();
                    (title, link)
                };
                let artist = artist.text().collect::<String>().into();
                let date = {
                    let date = date.text().collect::<String>();
                    (date != "0000/00/00")
                        .then(|| {
                            NaiveDate::parse_from_str(&date, "%Y/%m/%d")
                                .or_else(|_| NaiveDate::parse_from_str(&date, "%Y-%m-%d"))
                                .with_context(|| format!("While parsing date string {date:?}"))
                        })
                        .transpose()?
                        .into()
                };
                let material = material.text().collect::<String>().trim().to_owned().into();
                let song = song_list::Song {
                    genre,
                    title,
                    link,
                    artist,
                    date,
                    material,
                };
                res.songs.push(song);
            }
            ref tds => {
                error!("List of td (len={}):", tds.len());
                for td in tds {
                    error!("  - {}", td.html());
                }
                bail!(
                    "Invalid number of `td`s (expected 1 or 5, found {}), html: {}",
                    tds.len(),
                    tr.html(),
                );
            }
        }
    }

    serde_json::to_writer(fs_err::File::create(opts.output_json_path)?, &res)?;

    Ok(())
}
