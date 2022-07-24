use reqwest;
use clap::Parser;
use serde::{Deserialize, Serialize};
use csv::Writer;


#[derive(Parser)]
struct Cli {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct ConversationsListResponse {
    ok: bool,
    channels: Vec<Channel>,
}

#[derive(Serialize, Deserialize)]
struct Channel {
    id: String,
    name: String,
    is_channel: bool,
    is_group: bool,
    is_im: bool,
    is_mpim: bool,
    is_private: bool,
    is_archived: bool,
}

fn main() {
    let args = Cli::parse();

    let json_str = get_request_slack_api("conversations.list", &args.token);

    let res: ConversationsListResponse = serde_json::from_str(&json_str.text().unwrap()).unwrap();
    println!("{:?}", res.channels[0].name);
}

fn get_request_slack_api(method: &str, token: &str) -> reqwest::blocking::Response {
    let url = format!("https://slack.com/api/{}", method);

    let client = reqwest::blocking::Client::new();
    let resp = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send();

    return resp.unwrap();
}


fn write_csv(path: &str, records: Vec<Vec<&str>>) {
    let mut writer = Writer::from_path(path).unwrap();
    for record in records {
        writer.write_record(&record).unwrap();
    }
}
