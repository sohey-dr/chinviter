#![allow(unused)]

use clap::Parser;

#[derive(Parser)]
struct Cli {
    pattern: String,
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();
    let content = std::fs::read_to_string(&args.path)
        .expect("could not read file");

    for line in content.lines() {
        if line.contains(&args.pattern) {
            println!("{}", line);
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
