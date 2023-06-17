use std::io::{stdin, Write};

use outertube::{
    search::{ChannelResult, ShortResult, VideoResult},
    ClientBuilder,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = ClientBuilder::new().locale("en-US").build()?;

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return Err(anyhow::Error::msg("Usage: Provide one search query"));
    }
    let query = &args[0];

    let mut search_results = client.search(query).await?;
    println!(
        "Found {} results for \"{query}\". Estimated {} total results",
        search_results.videos.len() + search_results.shorts.len(),
        search_results.estimated_results
    );

    loop {
        for channel in &search_results.channels {
            let ChannelResult { id, name, url } = channel;
            println!("-- Channel --\n\nID: {id},\nName: {name},\nURL: {url}\n");
        }
        for video in &search_results.videos {
            let VideoResult {
                id,
                url,
                title,
                channel,
                view_count,
                thumbnails: _,
            } = video;
            println!("-- Video --\n\nID: {id},\nTitle: {title},\nChannel: {channel},\nURL: {url},\nviews: {view_count}\n");
        }

        for short in &search_results.shorts {
            let ShortResult {
                id,
                headline: title,
                url,
                thumbnails: _,
            } = short;

            println!("-- Short --\n\nID: {id},\nTitle: {title},\nURL: {url}\n");
        }


        // What if I want some more?

        print!("Continue finding results? [y/N]: ");
        std::io::stdout().flush()?;
        let mut should_continue = String::new();
        stdin().read_line(&mut should_continue)?;
        should_continue = should_continue.to_lowercase();
        if !(should_continue.contains('y') || should_continue.contains("yes")) {
            break;
        }

        // If the continuation couldn't occur becuase there were no results left.
        if !client.continue_search(&mut search_results).await? {
            println!("No more results!");
            break;
        }
    }
    Ok(())
}
