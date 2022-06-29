use serde::Deserialize;
use std::env;
use std::fmt;
use std::process;

macro_rules! pad {
    ($str:expr, $width:expr) => {
        &format!("{: <width$}", $str.replace("\n", " "), width = $width)
    };
}

#[derive(Default)]
struct Arguments {
    id: Option<String>,
    only: Option<String>,
}

#[derive(Deserialize)]
struct Result {
    #[serde(rename = "crate")]
    target: Option<Crate>,
}

#[derive(Deserialize)]
struct Crate {
    name: String,
    description: String,
    keywords: Vec<String>,
    max_stable_version: String,
    max_version: String,
    homepage: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
}

impl fmt::Display for Crate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines = Vec::new();
        lines.push(format!("{} {}", pad!("Name:", 15), pad!(self.name, 65)));
        lines.push(format!(
            "{} {}",
            pad!("Description:", 15),
            pad!(self.description, 65)
        ));
        lines.push(format!(
            "{} {}",
            pad!("Keywords:", 15),
            pad!(self.keywords[..].join(", "), 65)
        ));
        lines.push(format!(
            "{} {}",
            pad!("Stable Version:", 15),
            pad!(self.max_stable_version, 65)
        ));
        lines.push(format!(
            "{} {}",
            pad!("Latest Version:", 15),
            pad!(self.max_version, 65)
        ));
        lines.push(format!(
            "{} {}",
            pad!("Homepage:", 15),
            pad!(self.homepage.clone().unwrap_or_else(|| "- - -".into()), 65)
        ));
        lines.push(format!(
            "{} {}",
            pad!("Repository:", 15),
            pad!(
                self.repository.clone().unwrap_or_else(|| "- - -".into()),
                65
            )
        ));
        lines.push(format!(
            "{} {}",
            pad!("Documentation:", 15),
            pad!(
                self.documentation.clone().unwrap_or_else(|| "- - -".into()),
                65
            )
        ));
        write!(f, "{}", lines[..].join("\n"))
    }
}

#[tokio::main]
async fn main() -> surf::Result<()> {
    let args = get_args();
    if let Some(id) = args.id {
        let result = fetch_crate(&id).await?;
        if let Some(target) = result.target {
            if let Some(property) = args.only {
                match property.as_str() {
                    "latest" => println!("{}", target.max_version),
                    "stable" => println!("{}", target.max_stable_version),
                    "homepage" => println!("{}", target.homepage.unwrap_or_else(|| "- - -".into())),
                    "repository" => {
                        println!("{}", target.repository.unwrap_or_else(|| "- - -".into()))
                    }
                    "documentation" => {
                        println!("{}", target.documentation.unwrap_or_else(|| "- - -".into()))
                    }
                    _ => unreachable!(),
                }
            } else {
                println!("{}", target);
            }
        } else {
            println!("No crate found for '{}'!", &id);
        }
    } else {
        println!("Invalid arguments! Usage: crates <id>");
    }
    Ok(())
}

fn get_args() -> Arguments {
    let mut arguments = Arguments::default();
    let mut args = env::args();
    args.next();
    arguments.id = args.next();
    if let Some(arg) = args.next() {
        match arg.as_str() {
            "--latest" => arguments.only = Some("latest".into()),
            "--stable" => arguments.only = Some("stable".into()),
            "--homepage" => arguments.only = Some("homepage".into()),
            "--repo" => arguments.only = Some("repository".into()),
            "--doc" => arguments.only = Some("documentation".into()),
            _ => {
                println!("Invalid arguments! Usage: crates <id> [--latest, --stable, --homepage, --repo, --doc]");
                process::exit(1);
            }
        }
    }
    arguments
}

async fn fetch_crate(id: &str) -> surf::Result<Result> {
    let url = format!("https://crates.io/api/v1/crates/{}", &id);
    let mut response = surf::get(&url)
        .header("user-agent", "crates (github.com/typable/crates)")
        .await?;
    let result = response.body_json().await?;
    Ok(result)
}
