use reqwest;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Cli {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct Response {
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

    let resp = get_request_slack_api("conversations.list", &args.token);

    let res: Response = serde_json::from_str(&resp.text().unwrap()).unwrap();
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
