use anyhow::{anyhow, Result};
use colored::*;
use mime::Mime;
use reqwest::{header, Client, Response, Url};
use clap::{Parser, Subcommand, Args};
use std::{
    str::FromStr,
    collections::HashMap,
};

#[derive(Clone, Debug)]
struct KvPair {
    k: String,
    v: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok (Self {
            k: (split.next().ok_or_else(err)?.to_string()),
            v: (split.next().ok_or_else(err)?.to_string()),
        })
    }
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Opts {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Get(Get),
    Post(Post),
}

#[derive(Args, Debug)]
struct Get {
    #[arg(value_parser = parse_url)]
    url: String,
}

#[derive(Args, Debug)]
struct Post {
    #[arg(value_parser = parse_url)]
    url: String,
    #[arg(value_parser = parse_kv_pair)]
    body: Vec<KvPair>,
}

fn parse_url(s: &str) -> Result<String> {

    // 检查 Url 是否合法
    #[warn(unused_variables)]
    let _url: Url = s.parse()?;

//     Ok(s.into())
    Ok(s.to_string())
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }

    let resp = client.post(&args.url).json(&body).send().await?;
//     println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}

async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_header(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

fn get_content_type(resp: &Response) -> Option<Mime> {
       resp.headers()
           .get(header::CONTENT_TYPE)
           .map(|v| v.to_str().unwrap().parse().unwrap())
}

// print status
fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status)
}

fn print_header(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }

    print!("\n");
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
        }
        _ => println!("{}", body),

    }
}


#[tokio::main]
async fn main() -> Result<()> {
    let opt: Opts = Opts::parse();

    let mut headers = header::HeaderMap::new();
    headers.insert("X-POWERED-BY", "Rust".parse().unwrap());
    headers.insert(header::USER_AGENT, "Rust Httpie".parse().unwrap());
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    match &opt.command {
        Commands::Get(ref args) => {
            // https://blog.logrocket.com/understanding-rust-option-results-enums/
            // println!("{:#?}, {:#?} ", args, client);
            get(client, args).await?
        },
        Commands::Post(ref args) => {
            // println!("{:#?}", args);
            post(client, args).await?
        }
    };

    // println!("{:?}", opt);
    Ok(())
}