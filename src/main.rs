use reqwest;
use std::{thread, time};
use clap::Parser;
use serde::{Deserialize, Serialize};
use csv::Writer;


#[derive(Parser)]
struct Cli {
    subcommand: String,
    // In the current functionality, option is passed a token
    option: String,
}

#[derive(Serialize, Deserialize)]
struct ConversationsListResponse {
    ok: bool,
    channels: Vec<Channel>,
    response_metadata: ResponseMetadata,
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

#[derive(Serialize, Deserialize)]
struct ResponseMetadata {
    next_cursor: String,
}

const CONVERSATIONS_CSV_PATH: &str = ".bin/conversations.csv";

const API_COOL_TIME: time::Duration = time::Duration::from_secs(2);

fn get_request_slack_api(method: &str, token: &str) -> reqwest::blocking::Response {
    let url = format!("https://slack.com/api/{}", method);

    let client = reqwest::blocking::Client::new();
    let resp = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send();

    return resp.unwrap();
}

fn get_channels_from_slack(token: &str, next_cursor: String) -> (Vec<Vec<String>>, String) {
    let path = format!("conversations.list?types=private_channel&cursor={}", next_cursor);
    let json_str = get_request_slack_api(&path, token);
    let res: ConversationsListResponse = serde_json::from_str(&json_str.text().unwrap()).unwrap();
    let mut records: Vec<Vec<String>> = Vec::new();

    for channel in res.channels {
        let mut record: Vec<String> = Vec::new();
        record.push(channel.id);
        if channel.is_private {
            record.push("private".to_string());
        } else {
            record.push("public".to_string());
        }
        record.push(channel.name);

        records.push(record);
    }

    (records, res.response_metadata.next_cursor)
}

fn write_csv(path: &str, records: Vec<Vec<String>>) {
    let mut writer = Writer::from_path(path).unwrap();
    for record in records {
        writer.write_record(&record).unwrap();
    }
}

fn write_channels_to_csv(token: &str, next_cursor: String) {
    let (records, next_cursor) = get_channels_from_slack(token, next_cursor);
    write_csv(CONVERSATIONS_CSV_PATH, records);

    if next_cursor != "" {
        thread::sleep(API_COOL_TIME);
        write_channels_to_csv(token, next_cursor);
    }
}

fn set_up(args: Cli) {
    match args.subcommand.as_str() {
        "channels" => {
            write_channels_to_csv(&args.option, "".to_string());
        },
        _ => {
            println!("{}", args.subcommand);
        }
    }
}

fn main() {
    let args = Cli::parse();
    set_up(args);
}
