mod dto;

use dto::*;

use reqwest::Url;
use sqlx::postgres::PgPoolOptions;
use sqlx::query;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    // connect to the database
    let pool = PgPoolOptions::new()
        .connect("postgres://postgres:Damperstudent@localhost/wikilinks")
        .await
        .expect("Failed to connect to database");

    let mut queue = Vec::<String>::new();

    // select articles that do not have links and add them to the queue
    let articles = query!("SELECT title FROM articles WHERE cardinality(links) = 0 ORDER BY title desc")
        .fetch_all(&pool)
        .await
        .expect("Failed to load articles!")
        .into_iter()
        .map(|r| r.title);
    queue.extend(articles);

    // split the queue into 10 article chunks and clone the chunks (which are slices of Strings)
    // to a vector of chunks (now a vector of Strings)
    let queue = queue
        .chunks(10)
        .map(|c| c
            .iter()
            .map(|d| d.clone())
            .collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let queue = Arc::new(Mutex::new(queue));

    let (store_tx, mut store_rx) = tokio::sync::mpsc::unbounded_channel::<(String, Vec<String>)>();

    // In another thread, listen for messages from the other threads
    // to store the results of a call to the Wikipedia API. There's
    // no real benefit to doing it this way *with Postgres* because
    // the pool manages connections automatically and can be shared
    // between threads. A previous iteration used SQLite, which can
    // only be access from one thread at a time and would require
    // locking and unlock a Mutex for every write.
    tokio::spawn(async move {
        while let Some((name, links)) = store_rx.recv().await {
            println!("Updating {} = {}", name, links.len());

            query!("UPDATE articles SET links = $2 WHERE title = $1", name, links.as_slice())
                .execute(&pool)
                .await
                .expect("Failed to insert article");
        }
    });

    let mut threads = vec![];

    // spawn 5 threads to request links concurrently. significantly speeds up
    // the link fetching process, which can still take a long time.
    // Note: requests will begin to fail if too many threads are spawned (10+)
    for _ in 0..5 {
        let store_tx = store_tx.clone();
        let queue = Arc::clone(&queue);

        let thread = tokio::spawn(async move {
            // Each thread needs to use a reqwest Client to cache DNS requests or
            // requests will fail due to DNS server overload, especially if the server
            // is running on the user's network gateway
            let client = reqwest::Client::new();

            loop {
                let current = { queue.lock().await.pop() };
                if current.is_none() { break; }
                let current = current.unwrap();

                let mut r#continue = PageLinksContinue { pl_continue: None, r#continue: None };
                let mut collected = HashMap::new();

                loop {
                    let mut url = Url::parse("https://en.wikipedia.org/w/api.php?action=query&format=json&prop=links&formatversion=2&pllimit=max").unwrap();
                    url.query_pairs_mut().append_pair("titles", current.join("|").as_str());

                    let PageLinksContinue { r#continue: cont, pl_continue } = r#continue;

                    // continue parameters to get the next page of results

                    if let Some(ref pl_continue) = pl_continue {
                        url.query_pairs_mut().append_pair("plcontinue", pl_continue.as_str());
                    }

                    if let Some(ref cont) = cont {
                        url.query_pairs_mut().append_pair("continue", cont.as_str());
                    }

                    print!("Sending request cont={:?} @ {}...", pl_continue, current[0]);

                    let resp = client.get(url)
                        .header("user-agent", "qwright10 quentin.wright@ufl.edu / research @ University of Florida")
                        .send()
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap();

                    // if the request fails, we can exit and start again.
                    // progress continues from the last request
                    let resp: PageLinks = match serde_json::from_str(&resp) {
                        Ok(r) => r,
                        Err(_) => {
                            println!("========{}========", resp);
                            panic!();
                        }
                    };

                    // sleep to avoid request failure and spamming API
                    tokio::time::sleep(Duration::from_millis(50)).await;

                    // some pages will have no links
                    let links = match resp.query {
                        Some(links) => links,
                        None => {
                            break;
                        }
                    };

                    let links = links
                        .get("pages")
                        .unwrap();

                    for page in links {
                        let title = page.title.clone();
                        let l = page.links.as_ref()
                            .unwrap_or(&Vec::new())
                            .iter()
                            .map(|l| l.title.clone())
                            .collect::<Vec<_>>();

                        // println!("Collecting {} = {:?}", title, l);

                        let mut existing = collected.get_mut(&title);
                        if let None = existing {
                            collected.insert(title.clone(), Vec::new());
                            existing = Some(collected.get_mut(&title).unwrap());
                        }

                        existing.unwrap().extend(l);
                    }

                    // if there are more pages, continue requesting
                    if let Some(cont) = resp.r#continue {
                        r#continue = cont;
                    } else {
                        // otherwise break and continue with the next article in the queue
                        break;
                    }
                }

                for (title, links) in collected {
                    println!("{} contains {} links", title, links.len());

                    // send the results to the db writer thread above
                    store_tx.send((title, links)).unwrap();
                }
            }
        });

        threads.push(thread);

        // spawn threads 1s apart for no particular reason
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    // wait for all tasks to terminate
    for thread in threads {
        thread.await.unwrap();
    }
}