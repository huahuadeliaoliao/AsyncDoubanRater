[中文版](./README_zh.md) | English

# A-rust-crawler-for-crawling-user-ratings-of-movies
A rust crawler for crawling user ratings of movies

## Introduction

This project is a Rust-based web crawler specifically designed to scrape user ratings of movies from a popular movie database website, Douban. It fetches data like movie titles, details, ratings, and groups them as per user-defined categories. This crawler is built using modern Rust asynchronous programming paradigms, ensuring efficient and concurrent data processing.

## Features

- **Asynchronous Web Scraping**: Leverages Rust's powerful `async`/`await` syntax for non-blocking network requests.
- **Data Parsing**: Utilizes `scraper` for parsing HTML and extracting required information.
- **JSON Data Handling**: Employs `serde` for seamless serialization and deserialization of movie data.
- **Concurrent Task Execution**: Manages concurrent scraping tasks efficiently using `tokio::task`.
- **Error Handling**: Robust error handling to manage network and parsing errors gracefully.
- **File Operations**: Async file read and write operations for handling JSON data.