use std::{
    borrow::Cow,
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{Result, bail};
use derive_more::{Display, From};
use itertools::iterate;
use log::warn;
use percent_encoding::{CONTROLS, percent_encode};
use scraper::Html;
use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! selector {
    ($e: expr) => {{
        use ::scraper::Selector;
        use ::std::sync::LazyLock;
        static SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse($e).unwrap());
        &*SELECTOR
    }};
}

pub struct WikiFetcher {
    client: reqwest::blocking::Client,
    last_fetch: Option<Instant>,
    throttle: Duration,
}
impl Default for WikiFetcher {
    fn default() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            last_fetch: None,
            throttle: Duration::from_secs(1),
        }
    }
}

impl WikiFetcher {
    pub fn fetch(&mut self, title: PageTitle) -> Result<Html> {
        if let Some(last_fetch) = self.last_fetch
            && let Some(timeout) = self.throttle.checked_sub(Instant::now() - last_fetch)
        {
            sleep(timeout);
        }
        let retry_count = 5;
        for retry_timeout in iterate(1., |x| x * 2.).take(retry_count) {
            match (|| {
                let url = format!(
                    "https://music765plus.com/index.php?title={}&action=view",
                    percent_encode(title.0.as_bytes(), CONTROLS)
                );
                let response = self.client.get(&url).send()?;
                self.last_fetch = Some(Instant::now());
                if !response.status().is_success() {
                    bail!("Invalid status code: {}", response.status());
                }
                Ok(Html::parse_document(&response.text()?))
            })() {
                Ok(result) => return Ok(result),
                Err(e) => warn!("Fetch error: {e:#}"),
            }
            sleep(Duration::from_secs_f64(retry_timeout));
        }
        bail!("Failed after {retry_count} retries")
    }
}

#[derive(Debug, Display, From, Serialize, Deserialize)]
#[from(forward)]
pub struct PageTitle<'a>(Cow<'a, str>);

pub mod song_list {
    use chrono::NaiveDate;
    use derive_more::{Display, From};
    use serde::{Deserialize, Serialize};

    use crate::PageTitle;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct List {
        pub songs: Vec<Song>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Song {
        pub genre: GenreCode,
        pub title: SongTitle,
        pub artist: Artist,
        pub link: PageTitle<'static>,
        pub date: FirstAppearanceDate,
        pub material: FirstAppearanceMaterial,
    }

    #[derive(
        PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, From, Display,
    )]
    pub struct GenreCode(String);
    #[derive(Debug, Serialize, Deserialize, From, Display)]
    pub struct SongTitle(String);
    #[derive(Debug, Serialize, Deserialize, From, Display)]
    pub struct Artist(String);
    #[derive(Debug, Serialize, Deserialize, From)]
    pub struct FirstAppearanceDate(Option<NaiveDate>);
    #[derive(Debug, Serialize, Deserialize, From, Display)]
    pub struct FirstAppearanceMaterial(String);
}
