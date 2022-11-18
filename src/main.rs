use regex::Regex;
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use std::fs::OpenOptions;
use tokio;

#[derive(Serialize, Debug)]
struct MorningPaper {
    title: String,
    url: String,
    index: i32,
}

#[tokio::main]
async fn create_mp() -> Result<(), Box<dyn std::error::Error>> {
    let mut search_for_url =
        "https://blog.acolyer.org/2014/10/08/a-storm-drain-for-the-morning-paper/".to_string();
    // the last edition of The Morning Paper (as of 2021-02-08)
    let last_url = "https://blog.acolyer.org/2021/02/08/the-ants-and-the-pheromones/";
    // FIXME: very hardcoded/hacky to acolyer's website but does work
    // TIL you can have named capture groups, how cool!
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

    serde_json::to_writer(file, &data)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    create_mp()?;
    Ok(())
}
