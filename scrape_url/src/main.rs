use std::fs;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    for arg in std::env::args() {
    println!("{}", arg);
    }

    if args.len() < 2 {
        eprintln!("Usage: {} <url> <output_file>", args[0]);
        eprintln!("Default output file: outpout.md");
        std::process::exit(1);
    }

    let url = &args[1];
    // let output = &args[2];
    // let output = if args.len() > 2 {&args[2]} else  { "output.md" };
    let output = match args.get(2) {
        Some(output_file) => output_file,
        None => "output.md"
    };

    println!("Fetching {}", url);

    let body = reqwest::get(url).await?.text().await?;
    
    println!("Saving to {}", output);
    let md = html2md::parse_html(&body);

    fs::write(output, md.as_bytes())?;

    println!("Done. File has been save in {}", output);
    Ok(())
}