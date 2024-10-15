use reqwest::Client;
use clap::{Arg, Command};

pub async fn req(webpath: &str, endpoint: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();

    let response = client
    .get(&format!("{}{}", webpath, endpoint))
    .send()
    .await?;

    let status_code = response.status();
    let result_str: String = format!("{}{}: STATUS_CODE = {}", webpath, endpoint, status_code);

    Ok(result_str)
}

pub fn parse_args() -> (String, String, usize) {
    let matches = Command::new("request-app")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("HTTP Client for making requests")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .value_name("URL")
                .help("Specifies the base URL")
                .required(true)
                .num_args(1),
        )
        .arg(
            Arg::new("endpoint")
                .short('e')
                .long("endpoint")
                .value_name("ENDPOINT")
                .help("Specifies an endpoint or a file containing endpoints")
                .required(true)
                .num_args(1),
        )
        .arg(
            Arg::new("threads")
            .short('t')
            .long("threads")
            .value_name("THREADS")
            .help("Max threads")
            .required(false)
            .num_args(1)
            .value_parser(clap::value_parser!(usize)),
        )
        .get_matches();

    
    let webpath = matches.get_one::<String>("url").unwrap().to_string() + "/";
    let endpoint = matches.get_one::<String>("endpoint").unwrap().to_string();
    let threads = matches
        .get_one::<usize>("threads")
        .copied()
        .unwrap_or(1);
    
    if threads > 50 {
        eprintln!("Error: The number of threads must be between 1 and 50");
        std::process::exit(1);
    }

    (webpath, endpoint, threads)
}