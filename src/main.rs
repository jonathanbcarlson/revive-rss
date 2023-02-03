use atom_syndication::{EntryBuilder, FeedBuilder, LinkBuilder, TextBuilder};
use regex::Regex;
use reqwest::Client;
use serde::Serialize;
use serde_json::{json, ser::PrettyFormatter};
use std::fs::OpenOptions;
use tokio;

#[derive(Serialize, Debug)]
struct MorningPaper {
    title: String,
    url: String,
    index: i32,
}

#[tokio::main]
async fn create_mp_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut search_for_url =
        "https://blog.acolyer.org/2014/10/08/a-storm-drain-for-the-morning-paper/".to_string();
    // the last edition of The Morning Paper (as of 2021-02-08)
    let last_url = "https://blog.acolyer.org/2021/02/08/the-ants-and-the-pheromones/";
    // FIXME: very hardcoded/hacky to acolyer's website but does work
    let re = Regex::new(
        "<div class=\"nav-next\"><a href=\"(?P<url>.+)\" rel=\"next\">.+</span> (?P<title>.+)</a>",
    )?;
    let mut data = json!({"morning_papers": []});
    // TODO: add papers 1 to 44 from
    // https://blog.acolyer.org/2014/10/15/themorningpaper-reaches-50-papers/
    // from the link we also know the first is index 45
    let mut index = 45;

    let client = Client::new();

    loop {
        let content = client
            .get(search_for_url.clone())
            .send()
            .await?
            .text()
            .await?;
        let caps = re.captures(content.as_str()).unwrap();
        search_for_url = caps["url"].to_string();
        let title = caps["title"].to_string();

        let morning_paper = MorningPaper {
            title,
            url: search_for_url.clone(),
            index,
        };

        println!("mp {:#?}", morning_paper);

        let serialized = serde_json::to_value(morning_paper)?;

        data["morning_papers"]
            .as_array_mut()
            .unwrap()
            .push(serialized);

        index += 1;

        // TODO: could probably also have this break if don't match in capture group (as there's no Next)
        //       but will have this for now
        if search_for_url == last_url {
            break;
        }
    }

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("morning_papers.json")?;

    serde_json::to_writer_pretty(file, &data)?;

    Ok(())
}

fn create_mp_rss() -> Result<(), Box<dyn std::error::Error>> {
    let entry_url = LinkBuilder::default()
        .href("https://blog.acolyer.org/2014/10/08/outperforming-lru-with-an-adaptive-replacement-cache-algorithm/".to_string())
        .build();
    let entry_title = TextBuilder::default()
        .value("Outperforming LRU with an Adaptive Replacement Cache Algorithm".to_string())
        .build();
    let entry = EntryBuilder::default()
        .title(entry_title)
        .link(entry_url)
        .build();

    let link = LinkBuilder::default()
        .href("https://blog.acolyer.org".to_string())
        .build();
    let subtitle = TextBuilder::default()
        .value("Morning Paper written by Adrian Colyer".to_string())
        .build();

    let feed = FeedBuilder::default()
        .title("Morning Paper".to_string())
        .link(link)
        .subtitle(subtitle)
        .entries(vec![entry])
        .build();
    let rss_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("morning_paper_feed.xml")?;
    feed.write_to(rss_file)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: only create list of titles, urls if mp.json doesn't exist
    // create_mp_json()?;
    create_mp_rss()?;
    Ok(())
}
