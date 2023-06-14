pub use config::ClientVariant;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::{
    api,
    config::{self, Locale},
    error::Error,
    search::{SearchResult},
};


pub struct ClientBuilder {
    client_context: config::ClientContext,
    locale:         config::Locale,
}

impl ClientBuilder {
    pub fn new() -> Self { Self::default() }

    pub fn variant(mut self, variant: ClientVariant) -> Self {
        self.client_context = variant.into();
        self
    }

    /// Change the locale from the default `en-us`
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    pub fn build(self) -> Result<Client, Error> { Client::new(self.client_context, self.locale) }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            client_context: ClientVariant::default().into(),
            locale:         Locale::from("en-US"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    http_client:               reqwest::Client,
    pub(crate) client_context: config::ClientContext,
    pub(crate) locale:         Locale,
}

impl Client {
    fn new(client_context: config::ClientContext, locale: Locale) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate"));
        // headers.insert(
        //     "accept",
        //     HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*
        // ;q=0.8"), );
        // headers.insert(
        //     "accept-charset",
        //     HeaderValue::from_static("ISO-8859-1,utf-8;q=0.7,*;q=0.7"),
        // );

        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert(
            "Accept-Language",
            HeaderValue::from_str(&format!("{locale},{};q=0.5", locale.hl))
                .map_err(|e| Error::Unhandled(e.to_string()))?,
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert(
            "X-Youtube-Client-Name",
            HeaderValue::from_str(&client_context.id.to_string()).unwrap(),
        );
        headers.insert(
            "X-Youtube-Client-Version",
            HeaderValue::from_static(client_context.version),
        );
        headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("no-cors"));
        // headers.insert("cookie", HeaderValue::from_static("CONSENT=YES+"));
        if let Some(referer) = client_context.referer {
            headers.insert("Referer", HeaderValue::from_static(referer));
            headers.insert("Origin", HeaderValue::from_static(referer));
        } else {
            let referer = "https://www.youtube.com/";
            headers.insert("Referer", HeaderValue::from_static(referer));
            headers.insert("Origin", HeaderValue::from_static(referer));
        }

        let http_client = reqwest::ClientBuilder::new()
            .https_only(true)
            .default_headers(headers)
            .user_agent(
                client_context
                    .user_agent
                    .unwrap_or("Mozilla/5.0 (X11; Linux x86_64; rv:102.0) Gecko/20100101 Firefox/102.0"),
            )
            .gzip(true)
            .deflate(true)
            .build()?;

        //"X-Goog-Api-Format-Version": "1",
        //"X-YouTube-Client-Name": str(self.client_id),
        //"X-YouTube-Client-Version": self.client_version,
        //"User-Agent": self.user_agent,
        //"Referer": self.referer,
        //"Accept-Language": self.locale and self.locale.accept_language(),

        Ok(Self {
            client_context,
            http_client,
            locale,
        })
    }

    pub fn get_http_client(&self) -> reqwest::Client { self.http_client.clone() }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, Error> {
        SearchResult::from_search_results(api::endpoints::search(self, query, None).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_client_variant_try_from() {
        let max = ClientVariant::Tvhtml5ForKids as usize;
        assert!(ClientVariant::try_from(max + 1).is_err());
        for variant in 0..=max {
            ClientVariant::try_from(variant).unwrap();
        }
    }

    #[test]
    fn test_innertube_creation() { let _client = ClientBuilder::new().build().unwrap(); }

    #[tokio::test]
    async fn test_search() {
        let client = ClientBuilder::new().build().unwrap();
        let videos = client.search("Linus Tech Tips").await.unwrap();
        assert!(!videos.is_empty());
    }
}
