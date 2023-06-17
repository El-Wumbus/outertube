use serde::Deserialize;
use serde_json::Value;

use crate::{api::crawl_object_for_string, client::Client, endpoints, error::Error, util::AsciiStr};

const VIDEO_JSON_KEYS: &[&str] = &[
    "videoId",
    "thumbnail",
    "title",
    "viewCountText",
    "navigationEndpoint",
];

const SHORT_JSON_KEYS: &[&str] = &[
    "videoId",
    "thumbnail",
    "headline",
    "viewCountText",
    "navigationEndpoint",
];

const CHANNEL_JSON_KEYS: &[&str] = &["channelId", "title", "navigationEndpoint"];
const CHANNEL_INGORE_JSON: &[&str] = &["navigationEndpoint"];

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Thumbnail {
    pub url:    String,
    pub width:  Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResults {
    /// The total estimated number of results to be found.
    pub estimated_results: u64,
    /// Search refinements (corrections, similar searches, etc.)
    pub refinements:       Vec<String>,
    pub videos:            Vec<VideoResult>,
    pub shorts:            Vec<ShortResult>,
    pub channels:          Vec<ChannelResult>,

    /// The continuation parameter that allows for continuing the search
    continuation: String,
}

impl SearchResults {
    #[inline]
    pub(crate) async fn search(
        client: &Client,
        query: &str,
        params: Option<&str>,
    ) -> Result<SearchResults, Error> {
        // Todo: Figure out what the params are and their format.
        Self::from_search_results(&endpoints::search(client, query, params).await?)
    }

    fn from_search_results(results: &Value) -> Result<SearchResults, Error> {
        let contents = &results
            .get("contents")
            .ok_or(Error::JsonParse("No 'contents' found".to_string()))?;

        let refinements = serde_json::from_value(results["refinements"].clone()).unwrap_or_default();

        let continuation = crawl_for_continuation(contents)
            .map(str::to_string)
            .ok_or(Error::JsonParse("No continuation found".into()))?;
        let estimated_results = results["estimatedResults"]
            .as_str()
            .and_then(|x| x.parse::<u64>().ok())
            .ok_or(Error::JsonParse("No estimated result count found".into()))?;

        let mut results = SearchResults {
            estimated_results,
            refinements,
            continuation,
            videos: Vec::new(),
            shorts: Vec::new(),
            channels: Vec::new(),
        };

        // We'd run in to errors parsing nothing (or just junk "for you" results) if we don't stop
        // now.
        if estimated_results == 0 {
            return Ok(results);
        }

        let videos = crawl_for_objects_containing_keys(contents, VIDEO_JSON_KEYS, []);
        let shorts = crawl_for_objects_containing_keys(contents, SHORT_JSON_KEYS, ["title"]);
        let channels = crawl_for_objects_containing_keys(contents, CHANNEL_JSON_KEYS, ["videoId"]);

        for video in videos {
            results.videos.push(VideoResult::parse_video(video)?);
        }

        for short in shorts {
            results.shorts.push(ShortResult::parse_short(short)?);
        }

        for channel in channels {
            results.channels.push(ChannelResult::parse_channel(channel)?);
        }

        Ok(results)
    }

    /// Continue the search using the continuation, returns a bool indicating if any new
    /// results were able to be found.
    pub(crate) async fn continue_search(&mut self, client: &Client) -> Result<bool, Error> {
        let results = endpoints::search_continuation(client, &self.continuation).await?;
        let Some(x) = Self::from_continuation_search_results(&results)? else {return Ok(false)};

        if x.videos.is_empty() && x.shorts.is_empty() {
            return Ok(false);
        }

        self.videos = x.videos;
        self.shorts = x.shorts;
        self.estimated_results = x.estimated_results;
        self.continuation = x.continuation;

        Ok(true)
    }

    pub(crate) fn from_continuation_search_results(results: &Value) -> Result<Option<SearchResults>, Error> {
        let contents = results
            .get("onResponseReceivedCommands")
            .and_then(|x| x.get(0))
            .and_then(|x| x.get("appendContinuationItemsAction"))
            .and_then(|x| x.get("continuationItems"))
            .and_then(|x| x.get(0))
            .and_then(|x| x.get("itemSectionRenderer"))
            .and_then(|x| x.get("contents"))
            .or(crawl_for_contents(results, None));

        let estimated_results = results["estimatedResults"]
            .as_str()
            .and_then(|x| x.parse::<u64>().ok())
            .ok_or(Error::JsonParse("No estimated result count found".into()))?;

        let videos = crawl_for_objects_containing_keys(contents.unwrap_or(results), VIDEO_JSON_KEYS, []);
        let shorts =
            crawl_for_objects_containing_keys(contents.unwrap_or(results), SHORT_JSON_KEYS, ["title"]);
        let channels =
            crawl_for_objects_containing_keys(contents.unwrap_or(results), CHANNEL_JSON_KEYS, ["videoId"]);

        if videos.is_empty() && shorts.is_empty() && channels.is_empty() {
            return Ok(None);
        }

        let continuation = crawl_for_continuation(results)
            .map(str::to_string)
            .ok_or(Error::JsonParse("No continuation found".into()))?;

        let mut continuation_results = SearchResults {
            estimated_results,
            refinements: Vec::new(),
            continuation,
            videos: Vec::new(),
            shorts: Vec::new(),
            channels: Vec::new(),
        };

        for video in videos {
            continuation_results.videos.push(VideoResult::parse_video(video)?);
        }

        for short in shorts {
            continuation_results.shorts.push(ShortResult::parse_short(short)?);
        }

        for channel in channels {
            continuation_results
                .channels
                .push(ChannelResult::parse_channel(channel)?);
        }

        Ok(Some(continuation_results))
    }
}

fn crawl_for_contents<'a>(results: &'a Value, name: Option<&'a String>) -> Option<&'a Value> {
    if let Some(item) = results.as_object() {
        let item_i = item.get("contents");

        if let Some(item_i) = item_i {
            if item_i.is_array() {
                return Some(item_i);
            }
        } else {
            for (name, item) in item {
                let item = crawl_for_contents(item, Some(name));
                if item.is_some() {
                    return item;
                }
            }
        }
    } else if let Some(x) = results.as_array() {
        if let Some(name) = name {
            if name == "contents" {
                return Some(results);
            }
        }

        for item in x {
            let x = crawl_for_contents(item, None);
            if x.is_some() {
                return x;
            }
        }
    }

    None
}

/// Crawl search results for objects that contain or don't contain provided keys. Returns
/// a Vec of objects.
fn crawl_for_objects_containing_keys(
    contents: &Value,
    keys: impl AsRef<[&'static str]>,
    omit_keys: impl AsRef<[&'static str]>,
) -> Vec<&Value> {
    let keys = keys.as_ref();
    let omit_keys = omit_keys.as_ref();
    let mut found_objects = Vec::new();

    if let Some(contents_o) = contents.as_object() {
        // This is dumb
        let condition = keys.iter().all(|key| contents_o.contains_key(*key))
            && omit_keys.iter().all(|key| !contents_o.contains_key(*key));

        if condition {
            dbg!(&contents["videoId"]);
            found_objects.push(contents);
        } else {
            for (_, object) in contents_o {
                let mut videos = crawl_for_objects_containing_keys(object, keys, omit_keys);
                found_objects.append(&mut videos);
            }
        }
    } else if let Some(x) = contents.as_array() {
        for x in x {
            if let Some(x) = x.as_object() {
                for (_, x) in x {
                    let mut videos = crawl_for_objects_containing_keys(x, keys, omit_keys);
                    found_objects.append(&mut videos);
                }
            }
        }
    }

    found_objects
}

fn crawl_for_continuation(contents: &Value) -> Option<&str> {
    if let Some(contents) = contents.as_object() {
        let contents_s = contents
            .get("continuationCommand")
            .and_then(|x| x.get("token").and_then(Value::as_str));
        if let Some(x) = contents_s {
            return Some(x);
        }

        for (_, contents) in contents {
            let s = crawl_for_continuation(contents);
            if s.is_some() {
                return s;
            }
        }
    } else if let Some(x) = contents.as_array() {
        for x in x {
            let s = crawl_for_continuation(x);
            if s.is_some() {
                return s;
            }
        }
    }

    None
}

#[inline]
fn parse_url(val: &Value) -> Option<String> {
    val.get("navigationEndpoint")
        .and_then(|x| x.get("commandMetadata"))
        .and_then(|x| x.get("webCommandMetadata"))
        .and_then(|x| x.get("url").and_then(Value::as_str))
        .map(|path| "https://www.youtube.com".to_string() + path)
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoResult {
    pub id:         String,
    /// The watch url of the video
    pub url:        String,
    pub title:      String,
    pub channel:    String,
    /// May be inaccurate due to variance in responses from YouTube.
    pub view_count: u64,
    pub thumbnails: Vec<Thumbnail>,
}

impl VideoResult {
    fn parse_video(video: &Value) -> Result<Self, Error> {
        let id = video["videoId"]
            .as_str()
            .map(str::to_string)
            .ok_or(Error::JsonParse("No video id found".into()))?;

        let title = crawl_object_for_string(&video["title"], &["accessibility"])
            .ok_or(Error::JsonParse("No title found".into()))?
            .to_string();

        let channel = crawl_object_for_string(&video["longBylineText"], CHANNEL_INGORE_JSON)
            .or_else(|| crawl_object_for_string(&video["ownerText"], CHANNEL_INGORE_JSON))
            .or_else(|| crawl_object_for_string(&video["shortBylineText"], CHANNEL_INGORE_JSON))
            .ok_or(Error::JsonParse("No channel found".into()))?
            .to_string();

        let thumbnails = serde_json::from_value(
            video["thumbnail"]
                .get("thumbnails")
                .ok_or(Error::JsonParse("No thumbnails found".into()))?
                .clone(),
        )
        .map_err(|e| Error::JsonParse(e.to_string()))?;

        // Some youtube view counts are accurate and some just suck, so we have to parse them
        // differently despite them haviving a specific key for this other (worse) kind of count.
        // Additonally `simpleText` doesn't exist for what I assume are live streams.
        let view_count_text = video
            .get("viewCountText")
            .and_then(|x| {
                x.get("simpleText")
                    .or_else(|| x.get("runs").and_then(|x| x[0].get("text")))
                    .and_then(Value::as_str)
            })
            .ok_or(Error::JsonParse("No view count".into()))?;

        let view_count = match view_count_text {
            "No views" => 0,
            _ if view_count_text.ends_with_one_of(['K', 'M', 'B']) => {
                let x = match view_count_text {
                    _ if view_count_text.ends_with('K') => 1_000.0f64,
                    _ if view_count_text.ends_with('M') => 1_000_000.0f64,
                    _ if view_count_text.ends_with('B') => 1_000_000_000.0f64,
                    _ => unreachable!(),
                };

                let view_count_text = view_count_text
                    .remove_matches(['K', 'M', 'B'])
                    .replace_all(["views", "view"], "");
                let view_count_text = view_count_text.trim();
                let views = x * view_count_text.parse::<f64>().map_err(|e| {
                    Error::JsonParse(format!(
                        "Unparsable view count: \"{}\": {e}",
                        view_count_text.trim()
                    ))
                })?;

                views as u64
            }
            _ => {
                let view_count_text = view_count_text
                    .replace(',', "")
                    .replace_all(["views", "view"], "");
                let view_count_text = view_count_text.trim();
                view_count_text.parse::<u64>().map_err(|e| {
                    Error::JsonParse(format!("Unparsable view count: \"{}\": {e}", view_count_text))
                })?
            }
        };

        let url = parse_url(video).ok_or_else(|| Error::JsonParse("No watch URL".into()))?;

        Ok(Self {
            id,
            url,
            title,
            channel,
            view_count,
            thumbnails,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShortResult {
    pub id:         String,
    /// The title of the short.
    pub headline:   String,
    pub url:        String,
    pub thumbnails: Vec<Thumbnail>,
}

impl ShortResult {
    fn parse_short(short: &Value) -> Result<Self, Error> {
        let id = short["videoId"]
            .as_str()
            .map(str::to_string)
            .ok_or(Error::JsonParse("No video id found".into()))?;

        let headline = short["headline"]
            .get("simpleText")
            .and_then(Value::as_str)
            .ok_or(Error::JsonParse("No title found".into()))?
            .to_string();

        let thumbnails = serde_json::from_value(
            short["thumbnail"]
                .get("thumbnails")
                .ok_or(Error::JsonParse("No thumbnails found".into()))?
                .clone(),
        )
        .map_err(|e| Error::JsonParse(e.to_string()))?;

        let url = parse_url(short).ok_or_else(|| Error::JsonParse("No watch URL".into()))?;

        Ok(Self {
            id,
            headline,
            url,
            thumbnails,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelResult {
    pub id:   String,
    pub name: String,
    pub url:  String,
}

impl ChannelResult {
    fn parse_channel(channel: &Value) -> Result<Self, Error> {
        let id = channel["channelId"]
            .as_str()
            .map(str::to_string)
            .ok_or(Error::JsonParse("No video id found".into()))?;

        let name = channel["title"]
            .get("simpleText")
            .and_then(Value::as_str)
            .ok_or(Error::JsonParse("No title found".into()))?
            .to_string();

        let url = parse_url(channel).ok_or_else(|| Error::JsonParse("No channel URL".into()))?;

        Ok(Self { id, name, url })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{client::ClientBuilder, endpoints::search};

    #[tokio::test]
    async fn test_crawl_search_results() {
        let client = ClientBuilder::new().build().unwrap();
        let results = search(&client, "Linus Tech Tips", None).await.unwrap();
        let _videos = crawl_for_objects_containing_keys(&results["contents"], VIDEO_JSON_KEYS, []);
    }

    #[tokio::test]
    async fn test_parse_video() {
        let client = ClientBuilder::new().build().unwrap();
        let results = search(&client, "Linus Tech Tips", None).await.unwrap();
        let videos = crawl_for_objects_containing_keys(&results["contents"], VIDEO_JSON_KEYS, []);
        for video in videos {
            let _ = VideoResult::parse_video(video).unwrap();
        }
    }

    #[tokio::test]
    async fn test_parse_shorts() {
        let client = ClientBuilder::new().build().unwrap();
        let results = search(&client, "Linus Tech Tips", None).await.unwrap();
        let videos = crawl_for_objects_containing_keys(&results["contents"], SHORT_JSON_KEYS, ["title"]);
        for video in videos {
            let _ = ShortResult::parse_short(video).unwrap();
        }
    }

    #[tokio::test]
    async fn test_search_from_search_results() {
        let client = ClientBuilder::new().build().unwrap();
        let results = search(&client, "Linus Tech Tips", None).await.unwrap();
        let videos = SearchResults::from_search_results(&results).unwrap().videos;
        assert!(!videos.is_empty());
    }

    /// Warning: This test can take a long time if searching something with a lot of
    /// results as it will continue the search until it can't anymore.
    // #[tokio::test]
    async fn _test_search_continue() {
        let client = ClientBuilder::new().build().unwrap();
        let results = search(
            &client,
            "How Much Memory for 1,000,000 Threads in 7 Languages",
            None,
        )
        .await
        .unwrap();
        #[allow(unused_assignments)]
        let mut video_count = 0usize;
        let mut results = SearchResults::from_search_results(&results).unwrap();
        assert!(!results.videos.is_empty());
        video_count = results.videos.len();
        let inital_video_count = video_count;

        println!("Starting first continuation...");
        while results.continue_search(&client).await.unwrap() {
            assert!(!results.videos.is_empty());
            video_count += results.videos.len();
        }
        assert!(video_count > inital_video_count);
    }
}
