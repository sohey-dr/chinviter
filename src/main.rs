use reqwest;
use clap::Parser;

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
    match resp {
        Ok(resp) => {
            println!("{:?}", resp.text());
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn get_request_slack_api(method: &str, token: &str) -> std::result::Result<reqwest::blocking::Response, reqwest::Error> {
    let url = format!("https://slack.com/api/{}", method);

    let client = reqwest::blocking::Client::new();
    return client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send();
}
