use serde::Deserialize;
use serde_json::Value;

use crate::{api::crawl_object_for_string, error::Error};

#[derive(Debug, Clone, Deserialize)]
pub struct Thumbnail {
    pub url:    String,
    pub width:  u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id:         String,
    /// The watch url of the video
    pub url:        String,
    pub title:      String,
    pub channel:    String,
    pub view_count: u64,
    pub thumbnails: Vec<Thumbnail>,
}

impl SearchResult {
    pub(crate) fn from_search_results(results: Value) -> Result<Vec<Self>, Error> {
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

/// Crawl search results for video objects. Returns a Vec of video objects.
fn crawl_search_results(contents: &Value) -> Vec<&Value> {
    let mut found_video_objects = Vec::new();

    if let Some(contents_o) = contents.as_object() {
        let is_video = contents_o.contains_key("videoId")
            && contents_o.contains_key("thumbnail")
            && contents_o.contains_key("title")
            && contents_o.contains_key("viewCountText")
            && contents_o.contains_key("navigationEndpoint")
            && (contents_o.contains_key("longBylineText")
                || contents_o.contains_key("shortBylineText")
                || contents_o.contains_key("ownerText"));

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

fn parse_video(video: &Value) -> Result<SearchResult, Error> {
    let id = video
        .get("videoId")
        .and_then(|x| x.as_str().map(str::to_string))
        .ok_or(Error::JsonParse("No video id found".into()))?;

    let title = crawl_object_for_string(&video["title"], &["accessibility"])
        .ok_or(Error::JsonParse("No title found".into()))?
        .to_string();

    const CHANNEL_INGORE_JSON: &[&str] = &["navigationEndpoint"];
    let channel = crawl_object_for_string(&video["longBylineText"], CHANNEL_INGORE_JSON)
        .or_else(|| crawl_object_for_string(&video["ownerText"], CHANNEL_INGORE_JSON))
        .or_else(|| crawl_object_for_string(&video["shortBylineText"], CHANNEL_INGORE_JSON))
        .ok_or(Error::JsonParse("No title found".into()))?
        .to_string();

    let thumbnails = serde_json::from_value(video["thumbnail"]["thumbnails"].clone())
        .map_err(|e| Error::JsonParse(e.to_string()))?;

    let view_count = video["viewCountText"]["simpleText"]
        .as_str()
        .and_then(|x| x.replace(',', "").split_once(" views").map(|x| x.0.to_string()))
        .and_then(|x| x.parse::<u64>().ok())
        .ok_or(Error::JsonParse("No view count".into()))?;

    let url = video["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"]
        .as_str()
        .map(|path| "https://www.youtube.com".to_string() + path)
        .ok_or(Error::JsonParse("No watch url".into()))?;

    Ok(SearchResult {
        id,
        url,
        title,
        channel,
        thumbnails,
        view_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{client::ClientBuilder, endpoints::search};

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
        let videos = SearchResult::from_search_results(results).unwrap();
        assert!(!videos.is_empty())
    }
}
