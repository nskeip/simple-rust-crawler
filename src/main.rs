extern crate reqwest;
extern crate select;

use std::env;
use std::process::exit;

use reqwest::Url;

use select::document::Document;
use select::predicate::{Name};
use std::collections::HashSet;

#[tokio::main]
async fn main() {
    const MAX_HEIGHT: i8 = 1;

    let args: Vec<String> = env::args().collect();
    if args.iter().count() == 1 {
        println!("No url specified");
        exit(1);
    }

    let start_url = Url::parse(&args[1]).expect("Incorrect start url");
    let start_domain = start_url.domain().expect("Strange domain, you know?").to_string();
    let mut queue = vec![start_url];

    let mut visited_pages: HashSet<Url> = HashSet::new();

    let mut current_floor = 0;
    let mut siblings_on_current_floor = 1;
    let mut siblings_on_next_floor = 0;

    while current_floor <= MAX_HEIGHT && !queue.is_empty() {
        for _ in 0..siblings_on_current_floor {
            let url = queue.remove(0);
            visited_pages.insert(url.clone());

            let resp = reqwest::get(url.clone()).await;
            if resp.is_err() {
                println!("Failed to download {}", url);
                continue;
            }

            let text = resp.unwrap().text().await;
            if text.is_err() {
                println!("Failed to parse {}", url);
               continue;
            }

            let doc = Document::from(text.unwrap().as_str());

            for node in doc.select(Name("a")) {
                if let Some(href) = node.attr("href") {
                    if href.starts_with('#') {
                        continue;
                    }

                    if let Ok(new_absolute_url) = url.join(href) {
                        if let Some(new_domain) = new_absolute_url.domain() {
                            if new_domain.to_string() != start_domain {
                                continue;
                            }
                        }

                        if visited_pages.contains(&new_absolute_url.clone()) {
                            continue;
                        }
                        queue.push(new_absolute_url);
                        siblings_on_next_floor += 1;
                    }
                }
            };

        }
        current_floor += 1;
        siblings_on_current_floor = siblings_on_next_floor;
        siblings_on_next_floor = 0;
    }
}
