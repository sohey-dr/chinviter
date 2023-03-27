use clap::Parser;
use spinners::{Spinner, Spinners};
use rpassword::read_password;

use serde::{Deserialize, Serialize};

use reqwest;
use std::{thread, time, io};
use std::io::{stdout, Write};

use csv::{Reader, Writer};
use std::fs::OpenOptions;


#[derive(Parser)]
#[clap(
    name = "chinviter",
    author = "sohey",
    version = "v1.0.1",
    about = "CLI tool to invite Slack channels of a workspace"
)]
struct Cli {
    subcommand: String,

    #[clap(short = 'u', long = "user_id", default_value = "")]
    user_id: String,

    #[clap(short = 'f', long = "filter", default_value = "")]
    filter: String,
}

#[derive(Serialize, Deserialize)]
struct ConversationsInviteResponse {
    ok: bool,
    channel: Channel,
}

#[derive(Serialize, Deserialize)]
struct ConversationsListResponse {
    ok: bool,
    channels: Vec<Channel>,
    response_metadata: ResponseMetadata,
}


#[derive(Serialize, Deserialize)]
struct UserInfoResponse {
    ok: bool,
    user: Vec<User>,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: String,
    team_id: String,
    name: String,
    deleted: bool,
    color: String,
    real_name: String,
    tz: String, // e.g. Asia/Tokyo
    tz_label: String, // e.g. Pacific Daylight Time,
    profile: Profile,
    tz_offset: i32, // e.g. -25200,
    is_admin: bool,
    is_owner: bool,
    is_primary_owner: bool,
    is_restricted: bool,
    is_ultra_restricted: bool,
    is_bot: bool,
    updated: i32, // TODO: UnixTime Only
    is_app_user: bool,
    has_2fa: bool
}

#[derive(Serialize, Deserialize)]
struct Profile {
        avatar_hash: String,
        status_text: String,
        status_emoji: String,
        real_name: String,
        display_name: String,
        real_name_normalized: String,
        display_name_normalized: String,
        email: String, // TODO: email only validation
        image_original: String, // TODO: URL only validations
        image_24: String, // TODO: URL only validation
        image_32: String, // TODO: URL only validation
        image_48: String, // TODO: URL only validation
        image_72: String, // TODO: URL only validation
        image_192:String, // TODO: URL only validation
        image_512:String, // TODO: URL only validation
        team: String
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

const CONVERSATIONS_CSV_PATH: &str = ".tmp/conversations.csv";

const INVITE_TARGETS_CSV_PATH: &str = ".tmp/invite_targets.csv";

const API_COOL_TIME: time::Duration = time::Duration::from_secs(2);

fn get_request_slack_api(method: &str, token: &str) -> reqwest::blocking::Response {
    let url = format!("https://slack.com/api/{}", method);

    let client = reqwest::blocking::Client::new();
    let resp = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send();

    return resp.unwrap();
}

// https://api.slack.com/methods/users.info
fn get_user_info_from_slack(token: &str) -> String {
    let path = format!("users.info");
    let json_str = get_request_slack_api(&path, token);
    let res: UserInfoResponse = serde_json::from_str(&json_str.text().unwrap()).unwrap();
    let mut records: Vec<Vec<String>> = Vec::new();

    res.user.profile.email;
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

fn write_csv(path: &str, records: Vec<Vec<String>>) -> io::Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)
        .unwrap();

    let mut writer = Writer::from_writer(file);
    for record in records {
        writer.write_record(&record).unwrap();
    }

    writer.flush()
}

fn write_channels_to_csv(token: &str, next_cursor: String, filter: &str) -> Result<(), io::Error> {
    let (records, next_cursor) = get_channels_from_slack(token, next_cursor);

    // TODO: refactor
    let mut filtered_records = Vec::new();
    for record in records {
        if filter == "" || record[2].contains(filter) {
            filtered_records.push(record);
        }
    }

    write_csv(CONVERSATIONS_CSV_PATH, filtered_records)?;

    if next_cursor != "" {
        thread::sleep(API_COOL_TIME);
        write_channels_to_csv(token, next_cursor, filter)?;
    }
    Ok(())
}

fn duplicate_conversations_csv() -> io::Result<()> {
    let mut rdr = Reader::from_path(CONVERSATIONS_CSV_PATH).unwrap();
    let mut writer = Writer::from_path(INVITE_TARGETS_CSV_PATH).unwrap();

    for result in rdr.records() {
        let record = result.unwrap();
        let mut new_record: Vec<String> = Vec::new();

        // format: channel_id, channel_type, channel_name
        new_record.push(record.get(0).unwrap().to_string());
        new_record.push(record.get(1).unwrap().to_string());
        new_record.push(record.get(2).unwrap().to_string());

        writer.write_record(&new_record).unwrap();
    }

    Ok(())
}

fn invite_targets_to_slack(token: &str, user_id: &str) {
    let mut rdr = Reader::from_path(INVITE_TARGETS_CSV_PATH).unwrap();

    for record in rdr.records() {
        let record = record.unwrap();
        let channel_id = record.get(0).unwrap();

        let path = format!("conversations.invite?channel={}&users={}", channel_id, user_id);
        let json_str = get_request_slack_api(&path, token).text().unwrap();
        match serde_json::from_str(&json_str){
            Result::Ok(res) => {
                // TODO: refactor this
                match res {
                    ConversationsInviteResponse { ok, channel } => {
                        if ok {
                            println!("Invited {} to {}: {}", user_id, channel.name, channel.id);
                            stdout().flush().unwrap();
                        } else {
                            println!("result: {}", json_str);
                            println!("Failed to invite {} to {}", user_id, channel_id);
                        }
                    }
                }
            },
            Result::Err(err) => {
                println!("Failed to invite {} to {}", user_id, channel_id);
                println!("{}", err);
            }
        };

        thread::sleep(API_COOL_TIME);
    }
}

fn delete_invite_targets_csv() -> Result<(), io::Error> {
    std::fs::remove_file(CONVERSATIONS_CSV_PATH)?;
    Ok(())
}

fn set_up(args: Cli) -> Result<(), io::Error> {
    // let token = get_token();
    let token = "aaa".to_string();

    let mut sp = Spinner::new(Spinners::Dots9, "".into());
    match args.subcommand.as_str() {
        "channels" => {
            write_channels_to_csv(&token, "".to_string(), &args.filter)?;
        },
        "invite" => {
            if args.user_id == "" {
                println!("user_id is required");
                return Ok(());
            }

            let email_domain = option_env!("EMAIL_DOMAIN");
            println!("email_domain is set to {:?}", email_domain);

            if !email_domain.is_none() {
                // TODO: validationを実装
                let email = get_user_info_from_slack(token);
                if !email.ends_with(email_domain.unwrap()) {
                    println!("{} is not in {}", email, email_domain);
                    return Ok(());
                }
            }

            duplicate_conversations_csv()?;
            invite_targets_to_slack(&token, &args.user_id);
            delete_invite_targets_csv()?;
        },
        _ => {
            println!("{}: unknown command", args.subcommand);
            println!("Run 'chinviter help' for usage.")
        }
    }
    sp.stop();

    Ok(())
}

fn get_token() -> String {
    print!("token> ");
    stdout().flush().unwrap();
    let token = read_password().unwrap();

    token.trim().to_string()
}

fn main() -> Result<(), io::Error> {
    let args = Cli::parse();
    set_up(args)?;

    Ok(())
}
