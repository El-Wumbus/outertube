use std::fmt;

use serde_json::{json, Value};
use tracing::{event, Level};

use crate::{client::Client, config::CONFIG, error::Error};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum Endpoint {
    Config,
    Guide,
    Player,
    Browse,
    Search,
    Next,
    GetTranscript,
    MusicGetSearchSuggestions,
    MusicGetQueue,
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Endpoint::Config => "config",
            Endpoint::Guide => "guide",
            Endpoint::Player => "player",
            Endpoint::Browse => "browse",
            Endpoint::Search => "search",
            Endpoint::Next => "next",
            Endpoint::GetTranscript => "get_transcript",
            Endpoint::MusicGetSearchSuggestions => "music/get_search_suggestions",
            Endpoint::MusicGetQueue => "music/get_queue",
        };

        write!(f, "{s}")
    }
}

impl Endpoint {
    async fn post(self, client: &Client, data: Value) -> Result<Value, Error> {
        let endpoint = self.to_string();
        // let url = match client.client_context.api_key {
        //     Some(key) => format!("{}{endpoint}?key={key}&prettyPrint=false", CONFIG.base_url),
        //     None => format!("{}{endpoint}?prettyPrint=false", CONFIG.base_url),
        // };

        let url = if let Some(key) = client.client_context.api_key { format!("{}{endpoint}?key={key}&prettyPrint=false", CONFIG.base_url) } else { format!("{}{endpoint}?prettyPrint=false", CONFIG.base_url) };

        let post_result = client.get_http_client().post(url).json(&data).send().await;
        match post_result {
            Ok(response) => {
                event!(target: "innertube", Level::TRACE, "Successfully requested data from endpoint: {}", endpoint);

                let headers = response.headers();

                // Ensure
                if let Some(content_type) = headers.get("Content-Type") {
                    let content_type = content_type.to_str().map(str::to_lowercase).map_err(|_| {
                        Error::Unhandled("YouTube didn't return JSON when asked!".to_string())
                    })?;

                    let content_type_s = content_type
                        .split(';')
                        .map(str::to_string)
                        .find(|x| x == "application/json");

                    if content_type_s.is_none() {
                        return Err(Error::Unhandled(format!(
                            "YouTube returned \"{content_type}\" when the API asked for JSON!"
                        )));
                    }
                }

                match response.json::<Value>().await {
                    Ok(response_data) => {
                        if response_data["error"].is_null() {
                            Ok(response_data) 
                        } else {
                            Err(Error::YtRequest {
                                message: response_data["error"]["message"].to_string(),
                                endpoint,
                                request_data: data,
                            })
                        }
                    }
                    Err(e) => {
                        event!(target: "innertube", Level::ERROR, "Failed to extract json from response: {:?}", e);
                        Err(Error::YtRequest {
                            message:      e.to_string(),
                            endpoint:     endpoint.to_string(),
                            request_data: data,
                        })
                    }
                }
            }
            Err(e) => {
                tracing::event!(target: "intertube", Level::ERROR, "Request failed: {e:?}");

                Err(Error::YtRequest {
                    message:      e.to_string(),
                    endpoint:     endpoint.to_string(),
                    request_data: data,
                })
            }
        }
    }
}

fn make_yt_context(client: &Client) -> Value {
    let client_context = client.client_context;

    let mut context = json!({
        "hl": client.locale.hl,
        "clientName": client_context.name,
        "clientVersion": client_context.version,
    });

    if let Some(gl) = client.locale.gl.clone() {
        context["gl"] = serde_json::Value::String(gl);
    }

    json!({ "client": context })
}

/// Ok, so look. `YouTube` uses protobuf for sending their search parameters and I have no
/// way of getting their schema so proper parameters in the library cannot work. 
pub(crate) async fn search(client: &Client, query: &str, params: Option<&str>) -> Result<Value, Error> {
    let data = json! ({
        "query": urlencoding::encode(query),
        "context": make_yt_context(client),
        "params": params.unwrap_or(""),
    });

    Endpoint::Search.post(client, data).await
}

pub(crate) async fn search_continuation(client: &Client, continuation: &str) -> Result<Value, Error> {
    let data = json! ({
        "context": make_yt_context(client),
        "continuation": continuation,
    });

    Endpoint::Search.post(client, data).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientBuilder;

    #[tokio::test]
    async fn test_search() {
        let client = ClientBuilder::new().build().unwrap();
        let _x = search(&client, "Linus Tech Tips", None).await.unwrap();
    }
}
