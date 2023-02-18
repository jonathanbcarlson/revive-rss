use atom_syndication::{EntryBuilder, Feed, FeedBuilder, LinkBuilder, TextBuilder};
use chrono::DateTime;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, ser::PrettyFormatter};
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::path::Path;
use std::vec;
use tokio;

#[derive(Serialize, Debug, Deserialize)]
struct MorningPaper {
    title: String,
    url: String,
    index: i32,
}

#[derive(Serialize, Debug, Deserialize)]
struct MPFile {
    morning_papers: vec::Vec<MorningPaper>,
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

fn create_mp_rss(
    title: String,
    date: String,
    url: String,
    output_rss_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let author = atom_syndication::PersonBuilder::default()
        .name("Adrian Colyer".to_string())
        .build();

    let content = atom_syndication::ContentBuilder::default()
        .value(Some(title.to_string()))
        .build();

    let entry_url = LinkBuilder::default().href(url.to_string()).build();
    let entry_title = TextBuilder::default().value(title.to_string()).build();

    let date = DateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S%z").unwrap();

    let entry = EntryBuilder::default()
        .title(entry_title.clone())
        .link(entry_url.clone())
        .authors(vec![author.clone()])
        .id(url.to_string())
        .updated(date.clone())
        .content(content.clone())
        .build();

    let feed = FeedBuilder::default()
        .title("Morning Paper".to_string())
        .entries(vec![entry])
        .id("https://blog.acolyer.org/".to_string())
        .updated(date)
        .author(author)
        .icon(
            "https://secure.gravatar.com/blavatar/09326a066a08237015d6b84f026d36ae?s=32"
                .to_string(),
        )
        .build();

    let rss_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(output_rss_file)?;
    feed.write_to(rss_file)?;
    Ok(())
}

fn add_entry_to_mp_rss(title: String, date: String, url: String, output_rss_file: String) -> Feed {
    // if file exists, read it in, add entry, write it out
    let file = File::open(output_rss_file).unwrap();
    let mut feed = Feed::read_from(BufReader::new(file)).unwrap();

    let author = atom_syndication::PersonBuilder::default()
        .name("Adrian Colyer".to_string())
        .build();

    let content = atom_syndication::ContentBuilder::default()
        .value(Some(title.to_string()))
        .build();

    let entry_url = LinkBuilder::default().href(url.to_string()).build();
    let entry_title = TextBuilder::default().value(title.to_string()).build();

    let date = DateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S%z").unwrap();

    let entry = EntryBuilder::default()
        .title(entry_title.clone())
        .link(entry_url.clone())
        .authors(vec![author.clone()])
        .id(url.to_string())
        .updated(date.clone())
        .content(content.clone())
        .build();

    feed.entries.push(entry);
    feed
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_rss_file = "morning_paper_feed.xml";
    let mp_json = File::open("morning_papers.json")?;
    let reader = BufReader::new(mp_json);
    let mp_file: MPFile = serde_json::from_reader(reader).unwrap();

    for mp in mp_file.morning_papers {
        let title = mp.title;
        let url = mp.url;
        let re = Regex::new(
            "https://blog.acolyer.org/(?P<year>[0-9]{4})/(?P<month>[0-9]{2})/(?P<day>[0-9]{2}).*/",
        )?;
        let caps = re.captures(url.as_str()).unwrap();
        let year = caps["year"].to_string();
        let month = caps["month"].to_string();
        let day = caps["day"].to_string();
        let date = format!("{year}-{month}-{day}T00:00:00+00:00");
        if Path::new(output_rss_file).exists() {
            let feed = add_entry_to_mp_rss(title, date, url, output_rss_file.to_string());
            let file = File::create(output_rss_file).unwrap();
            feed.write_to(file)?;
        } else {
            create_mp_rss(title, date, url, output_rss_file.to_string())?;
        }
    }

    Ok(())
}
