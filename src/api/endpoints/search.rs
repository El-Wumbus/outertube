use serde_json::{json, Value};

use super::{make_context, Endpoint};
use crate::{error::Error, Client};

async fn search(client: &Client, query: &str, params: Option<&str>) -> Result<Value, Error> {
    let data = json! ({
        "query": query,
        "context": make_context(client),
        "params": params.unwrap_or(""),
    });

    Endpoint::Search.post(client, data).await
}

#[derive(Debug, Clone)]
struct Search {
    id:    String,
    title: String,
}

impl Search {
    fn from_search_results(results: Value) -> Result<Vec<Self>, Error> {
        let contents = &results["contents"];

        if !contents.is_object() {
            return Err(Error::Unhandled(
                "YouTube returned JSON I don't know how to parse".into(),
            ));
        };

        let mut videos = Vec::new();
        let results = crawl_search_results(contents);

        for video in results {
            videos.push(parse_video(video)?);
        }

        Ok(videos)
    }
}

/// Recursivley crawl search results for video objects. Returns a Vec of video objects.
fn crawl_search_results(contents: &Value) -> Vec<&Value> {
    let mut found_video_objects = Vec::new();

    if let Some(contents_o) = contents.as_object() {
        let is_video = contents_o.contains_key("videoId")
            && contents_o.contains_key("thumbnail")
            && contents_o.contains_key("title")
            && contents_o.contains_key("viewCountText");

        if !is_video {
            for (_, object) in contents_o {
                let mut videos = crawl_search_results(object);
                found_video_objects.append(&mut videos);
            }
        } else {
            found_video_objects.push(contents);
        }
    } else if let Some(x) = contents.as_array() {
        for x in x {
            if let Some(x) = x.as_object() {
                for (_, x) in x {
                    let mut videos = crawl_search_results(x);
                    found_video_objects.append(&mut videos);
                }
            }
        }
    }

    found_video_objects
}

fn parse_video(video: &Value) -> Result<Search, Error> {
    let id = video
        .get("videoId")
        .and_then(|x| x.as_str().map(str::to_string))
        .ok_or(Error::JsonParse)?;

    let title = crawl_object_for_string(&video["title"])
        .ok_or(Error::JsonParse)?
        .to_string();

    Ok(Search { id, title })
}

/// Recurses over an object and returns the first string it finds, or `None` if it never finds anything.
fn crawl_object_for_string(object: &Value) -> Option<&str> {
    if let Some(object) = object.as_object() {
        for (_, x) in object {
            if let Some(s) = x.as_str() {
                return Some(s);
            } else if x.is_object() {
                if let Some(x) = crawl_object_for_string(x) {
                    return Some(x);
                }
            } else {
                return None;
            }
        }
        None
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        client::ClientBuilder,
    };
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let client = ClientBuilder::new().build().unwrap();
        let _x = search(&client, "Linus Tech Tips", None).await.unwrap();
    }

    #[tokio::test]
    async fn test_crawl_search_results() {
        let client = ClientBuilder::new().build().unwrap();
        let results = search(&client, "Linus Tech Tips", None).await.unwrap();
        let _videos = crawl_search_results(&results["contents"]);
    }

    #[tokio::test]
    async fn test_parse_video() {
        let client = ClientBuilder::new().build().unwrap();
        let results = search(&client, "Linus Tech Tips", None).await.unwrap();
        let videos = crawl_search_results(&results["contents"]);
        for video in videos {
            let _ = parse_video(video).unwrap();
        }
    }

    #[tokio::test]
    async fn test_search_from_search_results() {
        let client = ClientBuilder::new().build().unwrap();
        let results = search(&client, "Linus Tech Tips", None).await.unwrap();
        let videos = Search::from_search_results(results).unwrap();
        dbg!(videos);
        panic!();
    }
}
