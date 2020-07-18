#![allow(box_pointers)]
#![allow(irrefutable_let_patterns)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_must_use)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_assignments)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use html5ever::tokenizer::{
    BufferQueue, Tag, TagKind, TagToken, Token, TokenSink, TokenSinkResult, Tokenizer,
    TokenizerOpts,
};
use std::borrow::Borrow;
use url::{ParseError, Url};

use async_std::task;
use surf;

type CrawlResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

type BoxFuture = std::pin::Pin<Box<dyn std::future::Future<Output = CrawlResult> + Send>>;

#[derive(Default, Debug)]
struct LinkQueue {
    links: Vec<String>,
}

impl TokenSink for &mut LinkQueue {
    type Handle = ();

    // <a href="link">some text</a>
    fn process_token(&mut self, token: Token, line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            TagToken(
                ref tag @ Tag {
                    kind: TagKind::StartTag,
                    ..
                },
            ) => {
                if tag.name.as_ref() == "a" {
                    for attribute in tag.attrs.iter() {
                        if attribute.name.local.as_ref() == "href" {
                            let url_str: &[u8] = attribute.value.borrow();
                            let s = String::from_utf8_lossy(url_str).into_owned();
                            self.links.push(s);
                            //if s.contains("/comments/") { self.links.push(s);}
                        }
                    }
                }
            }
            _ => {}
        }
        TokenSinkResult::Continue
    }
}

pub fn get_links(url: &Url, page: String) -> Vec<Url> {
    let mut domain_url = url.clone();
    domain_url.set_path("");
    domain_url.set_query(None);

    let mut buffer = BufferQueue::new();
    buffer.push_back(page.into());

    let mut queue = LinkQueue::default();
    let mut tokenizer = Tokenizer::new(&mut queue, TokenizerOpts::default());
    let _ = tokenizer.feed(&mut buffer);

    queue.links.sort();
    queue.links.dedup_by(|a, b| a.eq_ignore_ascii_case(b));

    queue
        .links
        .iter()
        .map(|link| match Url::parse(link) {
            Err(ParseError::RelativeUrlWithoutBase) => domain_url.join(link).unwrap(),
            Err(_) => panic!("Malformed link found: {}", link),
            Ok(mut url) => { url.set_query(None); url },
        })
        .collect()
}

fn do_something_with_the_body() {

}

fn box_crawl(pages: Vec<Url>, current: u8, max: u8) -> BoxFuture {
    Box::pin(crawl(pages, current, max))
}

async fn crawl(pages: Vec<Url>, current: u8, max: u8) -> CrawlResult {
    println!("Current Depth: {}, Max Depth: {}", current, max);

    if current > max {
        println!("Reached Max Depth");
        return Ok(());
    }

    let mut tasks = vec![];

    println!("\r -- crawling: {:?}", pages);

    for url in pages {
        let task = task::spawn(async move {
            println!("getting: {}", url);

            let mut res = surf::get(&url).await?;
            let body = res.body_string().await?;

            do_something_with_the_body();

            if current + 1 >= max { return Ok(()) }
            let links = get_links(&url, body);

            if links.is_empty() { return Ok(()) }

            println!("\r -- Following: {:?}", links);
            box_crawl(links, current + 1, max).await
        });
        tasks.push(task);
    }

    for task in tasks.into_iter() {
        task.await?;
    }

    Ok(())
}

fn main() -> CrawlResult {
    task::block_on(async {
        box_crawl(vec![Url::parse("https://www.reddit.com/").unwrap()], 0, 2).await
    })
}
