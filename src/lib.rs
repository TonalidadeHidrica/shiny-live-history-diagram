use std::{
    borrow::Cow,
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{Result, bail};
use derive_more::From;
use itertools::iterate;
use log::warn;
use percent_encoding::{CONTROLS, percent_encode};
use scraper::Html;

#[macro_export]
macro_rules! selector {
    ($e: expr) => {{
        use ::once_cell::sync::Lazy;
        use ::scraper::Selector;
        static SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse($e).unwrap());
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

#[derive(From)]
#[from(forward)]
pub struct PageTitle<'a>(Cow<'a, str>);
