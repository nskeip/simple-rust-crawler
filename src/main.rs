extern crate reqwest;
extern crate select;

use std::env;
use std::process::exit;

use select::document::Document;
use select::predicate::{Name};

use futures::executor::block_on;

// TODO: https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html
// TODO: https://blog.logrocket.com/a-practical-guide-to-async-in-rust/

#[tokio::main]
async fn main() {
    const MAX_HEIGHT: i8 = 5;

    let args: Vec<String> = env::args().collect();
    if args.iter().count() == 1 {
        println!("No url specified");
        exit(1);
    }

    let mut current_floor = 0;
    let mut queue = vec![&args[1]];  // queue with first url
    let mut siblings_on_current_floor = 1;

    while current_floor <= MAX_HEIGHT && !queue.is_empty() {
        for _ in 0..siblings_on_current_floor {
            let url = queue.remove(0);
            println!("{}", url);
            let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
            let doc = Document::from(resp.as_str());
            for node in doc.select(Name("a")) {
                println!("{}", node.attr("href").unwrap());
            };
        }
        current_floor += 1;
    }
}
