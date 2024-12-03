mod dto;

use dto::*;

use std::collections::HashMap;
use reqwest::Url;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{query, PgPool};

#[tokio::main]
async fn main() {
    /// Note: Because requests to the Wikipedia API for article titles
    /// use a generator, each subsequent request depends on the continue
    /// parameters for the previous request. We cannot request article titles
    /// concurrently, so everything is done on a single thread.

    // connect to the database
    let pool = PgPoolOptions::new()
        .connect("postgres://postgres:Damperstudent@localhost/wikilinks")
        .await
        .expect("Failed to connect to database");

    // select the most recent request so we can continue with the three continue params
    let existing_continue = query!("select continue as cont, pl_continue, gap_continue from requests order by created_at desc limit 1")
        .fetch_optional(&pool)
        .await
        .expect("Failed to fetch requests");

    // continue params for first request = existing_continue or None for all three params
    let mut r#continue = existing_continue
        .map(|c| PageLinksContinue {
            gap_continue: c.gap_continue,
            pl_continue: c.pl_continue,
            r#continue: c.cont,
        })
        .unwrap_or(PageLinksContinue { gap_continue: None, pl_continue: None, r#continue: None });

    // shared client to avoid overloading DNS server
    let client = reqwest::Client::new();

    loop {
        // fetch article titles
        let Ok(r) = fetch(&client, &pool, r#continue.clone()).await else { continue };
        let mut cont = r.1.unwrap_or(PageLinksContinue { gap_continue: None, pl_continue: None, r#continue: None });
        if r.0 {
            println!("Done");
            break;
        }

        // save continue values for next request or reuse values
        r#continue =  PageLinksContinue {
            gap_continue: cont.gap_continue.clone().or_else(|| r#continue.gap_continue.clone()),
            pl_continue: cont.pl_continue.clone(),
            r#continue: cont.r#continue.clone(),
        };
    }
}

pub async fn fetch(client: &reqwest::Client, pool: &PgPool, r#continue: PageLinksContinue) -> Result<(bool, Option<PageLinksContinue>), ()> {
    let mut url = Url::parse("https://en.wikipedia.org/w/api.php?action=query&format=json&generator=allpages&formatversion=2&plnamespace=0&pllimit=max&gapnamespace=0&gaplimit=max&gapdir=ascending&gapfilterredir=nonredirects").unwrap();

    // continue parameters to get the next page of results

    if let Some(ref cont) = r#continue.clone().r#continue {
        url.query_pairs_mut().append_pair("continue", &cont);
    }
    if let Some(ref gap_continue) = r#continue.clone().gap_continue {
        url.query_pairs_mut().append_pair("gapcontinue", gap_continue);
    }
    if let Some(ref pl_continue) = r#continue.clone().pl_continue {
        url.query_pairs_mut().append_pair("plcontinue", pl_continue);
    }

    print!("Sending request cont={:?}...", r#continue);

    let resp = match client.get(url)
        .header("User-Agent", "qwright10; wrightq00@gmail.com / research use")
        .send()
        .await
        .unwrap()
        .json::<PageLinks>()
        .await {
        Ok(r) => r,
        Err(_) => {
            eprintln!("Malformed response!");
            return Err(())
        }
    };

    println!("Done");

    // some pages will have no articles, but we should still continue
    let Some(links) = resp.query
        .and_then(|mut m| m.remove("pages")) else {
        eprintln!("Response missing pages!");
        return Ok((false, resp.r#continue));
    };

    // add successful request to requests table
    let _ = query!("INSERT INTO requests (pl_continue, gap_continue, continue) VALUES ($1, $2, $3)", r#continue.pl_continue, r#continue.gap_continue, r#continue.r#continue)
        .execute(pool)
        .await
        .map_err(|_| {
            eprintln!("Failed to insert continue");
        });

    for page in links {
        let title = page.title.clone();
        let l = page.links.as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|l| l.title.clone())
            .collect::<Vec<_>>();

        // store article title
        query!("INSERT INTO articles(title, links) VALUES ($1, $2) on conflict do nothing", title, l.as_slice())
            .execute(pool)
            .await
            .expect("Failed to insert article");
    }

    Ok((false, resp.r#continue))
}