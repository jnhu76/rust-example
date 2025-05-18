use std::str::FromStr;

use anyhow::{Ok, Result, anyhow};
use clap::{Args, Parser};
use colored::Colorize;
use mime::Mime;
use reqwest::{Client, Response, Url, header};

#[derive(Parser, Debug)]
#[command(name = "httpie")]
struct Opts {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Debug)]
enum Commands {
    Get(Get),
    Post(Post),
    // Put(Put),
    // Delete(Delete),
    // Head(Head),
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
    #[arg(value_parser = parse_kvpair)]
    body: Vec<KvPair>,
}

// #[derive(Args, Debug)]
// struct Put{
//     #[arg(value_parser = parse_url)]
//     url: String,
//     #[arg(value_parser = parse_kvpair)]
//     body: Vec<KvPair>,
// }

// #[derive(Args, Debug)]
// struct Delete{
//     #[arg(value_parser = parse_url)]
//     url: String,
// }
// #[derive(Args, Debug)]
// struct Head{
//     #[arg(value_parser = parse_url)]
//     url: String,
// }

fn parse_url(url: &str) -> Result<String> {
    let _url: Url = url.parse()?;

    Ok(url.into())
}

#[derive(Debug, Clone, PartialEq)]
struct KvPair {
    k: String,
    v: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!("Failed to parse {}", s);
        Ok(Self {
            k: split.next().ok_or_else(err)?.to_string(),
            v: split.next().ok_or_else(err)?.to_string(),
        })
    }
}

fn parse_kvpair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut form = std::collections::HashMap::new();
    for kv in &args.body {
        form.insert(kv.k.clone(), kv.v.clone());
    }

    let resp = client.post(&args.url).json(&form).send().await?;
    Ok(print_resp(resp).await?)
}

fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}", status);
}

fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }
    println!();
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
        }
        _ => {
            println!("{}", body)
        }
    }
}

async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);
    let mime = get_context_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

fn get_context_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    // print!("{:?}", opts);

    // headers
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("httpie/0.1.0"),
    );
    headers.insert("X-POWERED-BY", "Rust".parse()?);

    let client = Client::builder().default_headers(headers).build()?;

    match opts.command {
        Some(Commands::Get(args)) => get(client, &args).await?,
        Some(Commands::Post(args)) => post(client, &args).await?,
        None => {
            eprint!("No command given");
            std::process::exit(1);
        }
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_works() {
        assert!(parse_url("abc").is_err());
        assert!(parse_url("http://abc.xyz").is_ok());
        assert!(parse_url("https://httpbin.org/post").is_ok());
    }
    #[test]
    fn parse_kv_pair_works() {
        assert!(parse_kvpair("a").is_err());
        assert_eq!(
            parse_kvpair("a=1").unwrap(),
            KvPair {
                k: "a".into(),
                v: "1".into()
            }
        );
        assert_eq!(
            parse_kvpair("b=").unwrap(),
            KvPair {
                k: "b".into(),
                v: "".into()
            }
        );
    }
}
