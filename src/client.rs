pub use config::ClientVariant;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::{
    config::{self, Locale},
    error::Error,
    search::SearchResults,
};


pub struct ClientBuilder {
    client_context: config::ClientContext,
    locale:         config::Locale,
}

impl ClientBuilder {
    #[must_use]
    pub fn new() -> Self { Self::default() }

    #[must_use]
    pub fn variant(mut self, variant: ClientVariant) -> Self {
        self.client_context = variant.into();
        self
    }

    /// Change the locale from the default `en-US`
    #[must_use]
    pub fn locale(mut self, locale: impl Into<Locale>) -> Self {
        self.locale = locale.into();
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

        Ok(Self {
            http_client,
            client_context,
            locale,
        })
    }

    #[inline]
    #[must_use]
    pub fn get_http_client(&self) -> reqwest::Client { self.http_client.clone() }

    /// Search YouTube for a query and get videos, shorts, and channels as response.
    ///
    /// # Examples
    ///
    /// ```
    /// # use outertube::{ClientBuilder, error::Error};
    /// # tokio_test::block_on(async {
    /// # let client = ClientBuilder::new().build()?;
    /// let search_results = client.search("The TaiWAN Show").await?;
    /// assert!(!search_results.videos.is_empty());
    /// # Ok::<(), Error>(())
    /// # });
    /// ```
    #[inline]
    pub async fn search(&self, query: impl AsRef<str>) -> Result<SearchResults, Error> {
        SearchResults::search(self, query.as_ref(), None).await
    }

    /// Ok, so look. I think `YouTube` uses protobuf for sending their search parameters
    /// and I have no way of getting their schema so proper parameters in the library
    /// cannot work unless I were to go through and take note of every single
    /// combonation of parameters just for them to change it in a few weeks.
    ///
    /// To get the parameters you need for your specific search you can go to YT and
    /// select the combonation and grab the value of the URL parameter `sp` (e.g.
    /// `sp=EgQQARgD`). It must remain URL encoded when used as a parameter.
    ///
    /// # Example
    ///
    /// ```
    /// # use outertube::{ClientBuilder, error::Error};
    /// # tokio_test::block_on(async {
    /// # let client = ClientBuilder::new().build()?;
    /// // "EgQQARgD": Type = video + Duration = 4-20 minutes
    /// let search_results = client
    ///     .search_with_params("Linus Cat Tips", "EgQQARgD")
    ///     .await?;
    /// assert_eq!(
    ///     search_results.shorts.is_empty(),
    ///     search_results.channels.is_empty()
    /// );
    /// assert!(!search_results.videos.is_empty());
    /// # Ok::<(), Error>(())
    /// # });
    /// ```
    pub async fn search_with_params(
        &self,
        query: impl AsRef<str>,
        params: &str,
    ) -> Result<SearchResults, Error> {
        SearchResults::search(self, query.as_ref(), Some(params)).await
    }

    /// Continue searching. Returns `true` if more results were able to be found.
    ///
    /// # Example
    ///
    /// ```
    /// # use outertube::ClientBuilder;
    /// # tokio_test::block_on(async {
    /// # let client = ClientBuilder::new().build().unwrap();
    /// let mut search_results = client.search("Linus Cat Tips").await.unwrap();
    /// let original_videos = search_results.videos.clone();
    /// // Do something with the results ...
    /// if client.continue_search(&mut search_results).await.unwrap() {
    ///     // There's new results in there!
    ///     assert_ne!(original_videos, search_results.videos);
    /// }
    /// # });
    /// ```
    #[inline]
    pub async fn continue_search(&self, search: &mut SearchResults) -> Result<bool, Error> {
        search.continue_search(self).await
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
    fn test_client_creation() { let _client = ClientBuilder::new().build().unwrap(); }

    #[tokio::test]
    async fn test_search() {
        let client = ClientBuilder::new().build().unwrap();
        let results = client.search("Linus Tech Tips").await.unwrap();
        assert!(!results.videos.is_empty());
    }

    #[tokio::test]
    async fn test_search_continue() {
        let client = ClientBuilder::new().build().unwrap();
        let mut results = client.search("Linus Tech Tips").await.unwrap();
        assert!(!results.videos.is_empty());
        let previous_results = results.videos.clone();

        client.continue_search(&mut results).await.unwrap();
        assert!(!results.videos.is_empty());
        assert_ne!(previous_results, results.videos);
    }

    #[tokio::test]
    async fn test_search_with_params() {
        let client = ClientBuilder::new().build().unwrap();
        let search_results = client
            .search_with_params("Linus Cat Tips", "EgQQARgD")
            .await
            .unwrap();
        assert!(search_results.shorts.is_empty() && search_results.channels.is_empty());
        assert!(!search_results.videos.is_empty());
    }
}
