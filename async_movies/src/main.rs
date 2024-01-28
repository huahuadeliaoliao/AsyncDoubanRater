use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use futures::stream::{self, StreamExt};
use csv::{Writer, WriterBuilder};
use std::fs::OpenOptions;
use std::path::Path;
use std::io;
use csv;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Movie {
    group: String,
    title: String,
    details: String,
    rating: String,
}

async fn scrape_movie_data(client: &Client, url: &str, group: usize, page_limit: usize, all_movies: Arc<Mutex<Vec<Movie>>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut page_count = 0;
    let mut next_url = url.to_string();
    let file_name = format!("movies_group{}.csv", group);
    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_path(&file_name)?;
    writer.write_record(&["group", "title", "details", "rating"])?;
    while page_count < page_limit && !next_url.is_empty() {
        let response = client.get(&next_url).send().await?;
        if response.status().is_success() {
            let html = Html::parse_document(&response.text().await?);
            let item_selector = Selector::parse("div.item").unwrap();
            let title_selector = Selector::parse("li.title em").unwrap();
            let details_selector = Selector::parse("li.intro").unwrap();
            let rating_selector = Selector::parse("span[class^='rating']").unwrap();
            let next_selector = Selector::parse("span.next a").unwrap();

            for element in html.select(&item_selector) {
                let group = format!("user{}", group);

                let title = element.select(&title_selector)
                    .next()
                    .ok_or("title not found")?
                    .inner_html()
                    .trim()
                    .to_owned();

                let details = element.select(&details_selector)
                    .next()
                    .ok_or("details not found")?
                    .inner_html()
                    .trim()
                    .to_owned();

                let rating_class = element.select(&rating_selector).next()
                    .map(|e| e.value().attr("class").unwrap_or(""))
                    .unwrap_or("");
                let rating = if !rating_class.is_empty() {
                    rating_class.split('-').collect::<Vec<&str>>()[0].to_string()
                } else {
                    String::from("")
                };

                let movie = Movie {
                    group: group.clone(),
                    title: title.clone(),
                    details: details.clone(),
                    rating: rating.clone(),
                };

                all_movies.lock().unwrap().push(movie.clone());
                writer.serialize(movie)?;
            }

            let next_href = html.select(&next_selector).next()
                .map(|e| e.value().attr("href").unwrap_or(""))
                .unwrap_or("");
            next_url = format!("https://movie.douban.com{}", next_href);
            page_count += 1;
        } else {
            return Err(format!("request failed: {}", response.status()).into());
        }
    }
    writer.flush()?;
    Ok(())
}


async fn scrape_movies(urls: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut group = 1;
    let all_movies = Arc::new(Mutex::new(vec![]));
    let movie_futures: Vec<_> = urls.iter().map(|url| {
        let current_group = group;
        group += 1;
        scrape_movie_data(&client, url, current_group, 10, all_movies.clone())
    }).collect();

    let movies_results = stream::iter(movie_futures)
        .buffer_unordered(urls.len())
        .collect::<Vec<Result<(), Box<dyn std::error::Error>>>>()
        .await;

    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .from_writer(io::stdout());

    delete_files((1..=urls.len()).map(|i| format!("movies_group{}.csv", i)).collect())?;

    for movie_result in movies_results.into_iter() {
        match movie_result {
            Ok(()) => {
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    write_movies_to_csv(&all_movies, "movies.csv", &mut wtr)?;

    Ok(())
}

fn write_movies_to_csv(movies: &Arc<Mutex<Vec<Movie>>>, path: &str, _wtr: &mut Writer<io::Stdout>) -> Result<(), Box<dyn std::error::Error>> {
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);
    for movie in movies.lock().unwrap().iter() {
        writer.serialize(movie)?;
    }
    writer.flush()?;
    Ok(())
}

fn delete_files(paths: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    for path in paths {
        if Path::new(&path).exists() {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn read_urls_from_csv(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut urls: Vec<String> = vec![];
    for result in reader.records() {
        let record = result?;
        let url = record[0].to_string();
        urls.push(url);
    }
    Ok(urls)
}

#[tokio::main]
async fn main() {
    let urls = match read_urls_from_csv("collect.csv") {
        Ok(urls) => urls,
        Err(e) => {
            println!("error: {}", e);
            return;
        }
    };
    let urls_str: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
    if let Err(e) = scrape_movies(&urls_str).await {
        println!("error: {}", e);
    }
}