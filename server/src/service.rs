use http_body_util::BodyExt;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{header, Method, Request, Response};
use hyper::http::StatusCode;
use sqlx::{query, PgConnection, PgPool};
use crate::helpers::{empty, full};

pub async fn service(req: Request<Incoming>, articles: Arc<Vec<Vec<usize>>>, title_map: Arc<HashMap<String, usize>>, conn: Arc<PgPool>, rev_title_map: Arc<Vec<String>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let resp = Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(full(format!("{}", articles.len())))
                .unwrap();

            Ok(resp)
        }

        (&Method::POST, "/bfs") => {
            let body = req.collect().await?.to_bytes();
            let body = String::from_utf8_lossy(body.as_ref()).to_string();
            let endpoints = body.split("\n").collect::<Vec<_>>();
            let (start, end) = (endpoints[0].to_string(), endpoints[1].to_string());

            dbg!(&start, &end);

            let (Some(&start), Some(&end)) = (title_map.get(&start), title_map.get(&end)) else {
                return Ok(Response::builder()
                    .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .status(StatusCode::NOT_FOUND)
                    .body(empty())
                    .unwrap());
            };

            let mut queue = VecDeque::<usize>::with_capacity(articles.len());
            let mut visited = vec![false; articles.len()];
            let mut visit_count = 0;
            //let mut visited = HashSet::<i32>::with_capacity(articles.len());
            let mut predecessor = vec![0usize; articles.len()];
            //let mut predecessor = HashMap::<usize, usize>::with_capacity(articles.len());

            queue.push_back(start);
            visited[start] = true;

            while !queue.is_empty() {
                let curr = queue.pop_front().unwrap();

                if curr == end {
                    let mut curr = end;
                    let mut path = vec![rev_title_map[curr].clone()];
                    while curr != start {
                        curr = predecessor[curr];
                        path.push(rev_title_map[curr].clone());
                    }

                    path.reverse();

                    return Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .body(full(path.join("\n")))
                        .unwrap());
                }

                let adj = &articles[curr];

                for &link in adj {
                    if visited[link] { continue; }

                    predecessor[link] = curr;
                    visited[link] = true;
                    queue.push_back(link);
                    visit_count += 1;
                }

                if visit_count % 10 == 0 { println!("{}", visit_count); }
            }

            println!("depth= visited={} predecessors={:?}", visited.len(), predecessor.len());

            Ok(Response::builder()
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .status(StatusCode::IM_A_TEAPOT)
                .body(empty())
                .unwrap())
        }

        (&Method::POST, "/dfs") => {
            let body = req.collect().await?.to_bytes();
            let body = String::from_utf8_lossy(body.as_ref()).to_string();
            let endpoints = body.split("\n").collect::<Vec<_>>();
            let (start, end) = (endpoints[0].to_string(), endpoints[1].to_string());

            let (Some(&start), Some(&end)) = (title_map.get(&start), title_map.get(&end)) else {
                return Ok(Response::builder()
                    .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .status(StatusCode::NOT_FOUND)
                    .body(empty())
                    .unwrap());
            };

            let mut stack = Vec::<usize>::with_capacity(articles.len());
            let mut visited = vec![false; articles.len()];
            let mut visit_count = 0;
            let mut predecessor = vec![0usize; articles.len()];

            stack.push(start);
            visited[start] = true;

            while !stack.is_empty() {
                let curr = stack.pop().unwrap();

                if curr == end {
                    let mut curr = end;
                    let mut path = vec![rev_title_map[curr].clone()];
                    while curr != start {
                        curr = predecessor[curr];
                        path.push(rev_title_map[curr].clone());
                    }

                    path.reverse();

                    return Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .body(full(path.join("\n")))
                        .unwrap());
                }

                let adj = &articles[curr];

                for &link in adj {
                    if visited[link] { continue }

                    predecessor[link] = curr;
                    visited[link] = true;
                    stack.push(link);
                    visit_count += 1;
                }

                if visit_count % 10 == 0 { println!("{}", visit_count) }
            }

            println!("visited={}", visit_count);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(empty())
                .unwrap())
        }

        (&Method::POST, "/search") => {
            let query = req.collect().await?.to_bytes();
            let query = String::from_utf8_lossy(query.as_ref()).to_string();

            let results = query!("SELECT title FROM articles WHERE lower(title) LIKE $1 ORDER BY length(title) ASC, title ASC LIMIT 10", query.to_lowercase())
                .fetch_all(&*conn)
                .await
                .unwrap()
                .into_iter()
                .map(|r| r.title)
                .collect::<Vec<_>>()
                .join("\n");

            let resp = Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(full(results))
                .unwrap();

            Ok(resp)
        }

        _ => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(empty())
                .unwrap())
        }
    }
}