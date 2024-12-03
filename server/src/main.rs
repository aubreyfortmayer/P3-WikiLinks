mod service;
mod helpers;

use std::collections::HashMap;
use std::error::Error;
use std::net::{SocketAddr};
use tokio::net::TcpListener;
use std::str::FromStr;
use std::sync::Arc;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use hyper::server::conn::http1;
use sqlx::postgres::PgPoolOptions;
use sqlx::query;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let bind_addr = SocketAddr::from_str("0.0.0.0:3000").unwrap();
    let listener = TcpListener::bind(bind_addr).await?;

    println!("Listening on {}, pid = {}", bind_addr, std::process::id());

    let conn = PgPoolOptions::new()
        .connect("postgres://postgres:password@localhost:5432/wikilinks")
        .await
        .expect("Failed to connect to database");

    let articles = query!("SELECT id, title, condensed_links FROM articles ORDER BY id ASC")
        .fetch_all(&conn)
        .await
        .expect("Failed to load articles");
    
    let mut graph = vec![vec![]; articles.len() + 1];
    let mut title_map = HashMap::<String, usize>::with_capacity(articles.len());
    let mut rev_title_map = vec![String::new(); articles.len() + 1];
    
    for article in articles {
        graph[article.id as usize] = article.condensed_links.into_iter().map(|v| v as usize).collect::<Vec<_>>();
        rev_title_map[article.id as usize] = article.title.clone();
        title_map.insert(article.title, article.id as usize);
    }
    
    println!("Loaded {} articles into graph", graph.len());
    
    let conn = Arc::new(conn);
    let graph = Arc::new(graph);
    let title_map = Arc::new(title_map);
    let rev_title_map = Arc::new(rev_title_map);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let conn = Arc::clone(&conn);
        let graph = Arc::clone(&graph);
        let title_map = Arc::clone(&title_map);
        let rev_title_map = Arc::clone(&rev_title_map);

        tokio::task::spawn(async move {
           let svc = service_fn(|req| service::service(req, Arc::clone(&graph), Arc::clone(&title_map), Arc::clone(&conn), Arc::clone(&rev_title_map)));

            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("Server error: {:?}", err);
            }
        });
    }
}