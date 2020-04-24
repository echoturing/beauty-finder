use std::collections::HashMap;
use std::fs;

use lazy_static::*;
use libxml::parser::*;
use libxml::xpath::Context;
use reqwest;
use tokio;

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.163 Safari/537.36";

const SITE_MAP: &str = "https://www.vmgirls.com/sitemap.shtml";


lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}



pub async fn run() {
    let site_map = get_site_map_data().await.unwrap();
    for url in site_map {
        let images = parse_current_images(async_do_request(&url).await.unwrap()).unwrap();
        println!("{:?}", images);
        for image in images {
            tokio::spawn(download(image));
        }
    }
}


async fn download(url: String) {
    println!("now download {}", url);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, UA.parse().unwrap());

    let parsed_url = urlparse::urlparse(&url);
    let file_path = format!("{}{}", "images/", get_path_last(&parsed_url.path));

    if std::path::Path::new(&file_path).exists() {
        println!("file already exist");
        return;
    }
    let image_data = reqwest::Client::new()
        .get(&url)
        .headers(headers)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();


    tokio::fs::write(file_path, image_data).await;
}

fn get_path_last(path: &str) -> String {
    let splits: Vec<&str> = path.split('/').collect();
    splits[splits.len() - 1].to_string()
}


async fn get_site_map_data() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let site_map_list = parse_site_map(async_do_request(SITE_MAP).await.unwrap()).unwrap();
    Ok(site_map_list)
}


async fn async_do_request(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, UA.parse().unwrap());
    let resp = HTTP_CLIENT
        .get(url)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;
    Ok(resp)
}


fn parse_site_map(html: String) -> Result<Vec<String>, failure::Error> {
    let parser = libxml::parser::Parser { format: ParseFormat::HTML };
    let document = parser.parse_string(html).unwrap();
    let context = Context::new(&document).unwrap();
    let url_nodes = context.evaluate("//div[@id='content']/ul/li/a").unwrap().get_readonly_nodes_as_vec();
    let mut result = vec![];
    for node in url_nodes {
        result.push(node.get_attribute("href").unwrap())
    }
    Ok(result)
}

fn parse_current_images(html: String) -> Result<Vec<String>, failure::Error> {
    let parser = libxml::parser::Parser { format: ParseFormat::HTML };
    let document = parser.parse_string(html).unwrap();
    let context = Context::new(&document).unwrap();
    let image_nodes = context.evaluate(&format!("//img")).unwrap().get_readonly_nodes_as_vec();
    let title = context.evaluate("//title").unwrap().get_readonly_nodes_as_vec().get(0).unwrap().get_content();
    let title_splits: Vec<&str> = title.split("丨").map(|x| x.trim()).collect();
    let mut result = vec![];
    for node in image_nodes {
        if let Some(src) = node.get_attribute("data-src") {
            if let Some(alt) = node.get_attribute("alt") {
                if alt.contains(title_splits[0]) {
                    result.push(src)
                }
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::crawler::{async_do_request, parse_current_images, parse_site_map};

    #[tokio::test]
    async fn test_get_html() {
        if let Ok(data) = async_do_request("http://172.17.87.155:27015/internal/api/v1/items").await {
            // println!("{}", data);
        }
    }

    #[test]
    fn test_parse_current_images() {
        use std::fs;
        let data = fs::read_to_string("test_data/detail.html").unwrap();
        let images = parse_current_images(data).unwrap();
        println!("images:{:?}", images)
    }

    #[test]
    fn test_parse_site_map() {
        use std::fs;
        let data = fs::read_to_string("test_data/site_map.html").unwrap();
        let urls = parse_site_map(data).unwrap();
        println!("site_maps:{:?}", urls)
    }
}




