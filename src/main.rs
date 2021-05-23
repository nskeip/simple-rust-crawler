extern crate async_std;
extern crate reqwest;
extern crate select;

use std::env;
use std::collections::HashSet;
use std::process::exit;

use reqwest::{Url, Error};

use futures::future::{join_all};
use select::document::Document;
use select::predicate::{Name};

use tokio::runtime::Runtime;

use async_std::{
    prelude::*,
    task,
};

const MAX_HEIGHT: i8 = 2;

async fn process_url(url: Url) -> Result<(String, Vec<String>), Error> {
    let resp = reqwest::get(url.clone()).await?;
    let text = resp.text().await?;
    let doc = Document::from(text.as_str());
    let mut resulting_vector = Vec::new();
    for node in doc.select(Name("a")) {
        if let Some(href) = node.attr("href") {
            resulting_vector.push(href.to_string());
        }
    }
    Ok((text, resulting_vector))
}

async fn run(start_url: Url) -> Result<(), Error> {
    let start_domain = start_url
        .domain()
        .expect("Strange domain, you know?");

    let mut current_floor_queue: Vec<Url>;
    let mut next_floor_queue = vec![start_url.clone()];
    let mut visited_pages: HashSet<Url> = HashSet::new();

    let mut current_floor = 0;
    let mut siblings_on_current_floor = 1;

    while current_floor <= MAX_HEIGHT {
        current_floor_queue = next_floor_queue;
        next_floor_queue = Vec::new();

        let mut proc_url_futures =
            Vec::with_capacity(siblings_on_current_floor);

        // собираем все подряд
        for url in current_floor_queue.clone() {
            if visited_pages.contains(&url) {
                continue;
            }
            visited_pages.insert(url.clone());
            proc_url_futures.push(process_url(url));
        }

        let proc_results = futures::future::join_all(proc_url_futures).await;
        for (i, r) in proc_results.iter().enumerate() {
            if let Ok((page_text, page_links)) = r {
                for href in page_links {
                    if href.starts_with('#') {
                        continue;
                    }

                    let url = current_floor_queue[i].clone();

                    if let Ok(new_absolute_url) = url.join(href) {
                        if let Some(new_domain) = new_absolute_url.domain() {
                            if new_domain.to_string() != start_domain {
                                continue;
                            }
                        }
                        next_floor_queue.push(new_absolute_url);
                    }
                }
            }
        }

        current_floor += 1;
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().count() == 1 {
        println!("No url specified");
        exit(1);
    }

    let start_url = Url::parse(&args[1]).expect("Incorrect start url");

    Runtime::new().expect("Failed to create runtime").block_on(run(start_url));
}
