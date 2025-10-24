use std::{array, collections::HashSet, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use fs_err::File;
use itertools::Itertools;
use scraper::{ElementRef, Html, Node};
use shiny_live_history_diagram::{
    parse_date_permissive, selector,
    song_details::{
        self,
        performance::{Performance, Performer},
    },
    song_list::{self, GenreCode},
};

#[derive(Parser)]
struct Opts {
    genres: Vec<GenreCode>,

    #[arg(long, default_value = "./ignore/list.json")]
    list_json_path: PathBuf,
    #[arg(long, default_value = "./ignore/wiki_html/")]
    wiki_html_dir_path: PathBuf,
    #[arg(long, default_value = "./ignore/song_list.json")]
    output_json_path: PathBuf,
}

fn main() -> Result<()> {
    env_logger::builder().format_timestamp_nanos().init();
    let opts = Opts::parse();

    let list: song_list::List = serde_json::from_reader(File::open(opts.list_json_path)?)?;
    let genres = HashSet::<_>::from_iter(&opts.genres);

    let mut songs = vec![];

    for song in &list.songs {
        if !genres.contains(&song.genre) {
            continue;
        }
        let path = opts.wiki_html_dir_path.join(song.link.to_file_name()?);
        let html = Html::parse_document(&fs_err::read_to_string(path)?);

        let mut performances: Vec<Performance> = vec![];

        for (tr_former, tr_latter) in html.select(selector!("table.InfoboxLive3 tr")).tuples() {
            performances.push(parse_performance([tr_former, tr_latter]).with_context(|| {
                format!(
                    "While parsing {}: {}{}",
                    song.title,
                    tr_former.html(),
                    tr_latter.html()
                )
            })?);
        }

        songs.push(song_details::Song {
            title: song.title.clone(),
            performances,
        });
    }

    serde_json::to_writer(File::create(opts.output_json_path)?, &songs)?;

    Ok(())
}

fn parse_performance([tr_former, tr_latter]: [ElementRef; 2]) -> Result<Performance> {
    let mut tds = tr_former.select(selector!("td"));
    let (date, kind) = {
        let td = tds
            .next()
            .with_context(|| format!("Missing first `td`: {}", tr_former.html()))?;
        let [date, kind] = split_by_br(td)?;
        let date = parse_date_permissive(&date)
            .with_context(|| format!("Failed to parse date string {date:?}"))?
            .into();
        (date, kind.into())
    };
    let (concert_name, venue) = {
        let td = tds
            .next()
            .with_context(|| format!("Missing second `td`: {}", tr_former.html()))?;
        let [concert_name, venue] = split_by_br(td)?;
        (concert_name.into(), venue.into())
    };

    let performers = parse_performers(&tr_latter.text().collect::<String>())?;

    Ok(Performance {
        date,
        kind,
        concert_name,
        venue,
        performers,
    })
}

fn split_by_br(e: ElementRef) -> Result<[String; 2]> {
    let mut ret = array::from_fn(|_| String::new());
    let mut i = 0;
    for child in e.children() {
        if let Some(e) = ElementRef::wrap(child) {
            if e.value().name() == "br" {
                i += 1;
            } else {
                for text in e.text() {
                    ret[i] += text;
                }
            }
        } else if let Node::Text(text) = child.value() {
            ret[i] += text;
        }
        if i >= 2 {
            bail!("Two or more `<br>` tag found: {}", e.html());
        }
    }
    if i != 1 {
        bail!("No `<br>` tag found: {}", e.html());
    }
    Ok(ret)
}

fn parse_performers(s: &str) -> Result<Vec<Performer>> {
    let s = s.trim();
    log::info!("{s}");
    // TODO implement!
    Ok(vec![])
}
