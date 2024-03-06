#![feature(let_chains)]
mod decorators;

use decorators::*;

use bytes::Buf;
use chrono::{DateTime, Utc};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Debug, Copy, Clone)]
enum Decorator {
    Markdown,
    Plain,
}

#[derive(ValueEnum, Debug, Copy, Clone)]
enum OutputFormat {
    Json,
    Pretty,
    Notification,
}

/// ChromeOS Releases commandline.
///
/// Fetches the Chrome Releases feed and filters to only chromeOS updates.
#[derive(Parser)]
#[command(version = clap::crate_version!())]
struct Cli {
    /// Decorator to format the release HTML with.
    #[arg(short = 'D', long, value_enum, default_value_t = Decorator::Markdown)]
    decorator: Decorator,

    /// Format to print releases in.
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Pretty)]
    format: OutputFormat,

    /// Disable filtering the release HTML to remove boilerplate.
    #[arg(short = 'F', long = "no-filter")]
    unfiltered: bool,

    /// Number of releases to fetch from the feed. This does NOT correspond to the number of
    /// releases returned.
    ///
    /// This may not work as you expect. The feed caps the number at some unknown max value.
    #[arg(short, long, default_value_t = 25)]
    releases: u32,

    /// Start index of releases to fetch from the feed. This does NOT correspond to the number of
    /// releases returned.
    #[arg(short, long, default_value_t = 1)]
    start: u32,

    /// Store and use a timestamp to only track new releases.
    ///
    /// The timestamp is stored in the XDG Cache Directory in the folder crosreleasenotifier.
    #[arg(short, long)]
    diff: bool,
}

fn html2md(html: String, decorator: Decorator) -> String {
    match decorator {
        Decorator::Markdown => {
            let decorator = MdDecorator::new();
            html2text::from_read_with_decorator(html.as_bytes(), usize::MAX, decorator)
        }
        Decorator::Plain => {
            let decorator = PlainDecorator::new();
            html2text::from_read_with_decorator(html.as_bytes(), usize::MAX, decorator)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Release {
    title: String,
    summary: String,
    content: String,
    timestamp: DateTime<Utc>,
}

async fn get_releases(opts: &Cli) -> Result<Vec<Release>, Box<dyn std::error::Error>> {
    let body = reqwest::get(format!(
        "https://www.blogger.com/feeds/8982037438137564684/posts/default?start-index={}&max-results={}",
        opts.start,
        opts.releases
    ))
    .await?
    .bytes()
    .await?
    .reader();
    let feed = feed_rs::parser::parse(body)?;
    Ok(feed
        .entries
        .into_iter()
        .filter(|x| {
            x.categories.iter().map(|x| &x.term).any(|x| {
                x == "ChromeOS" || x == "Chrome OS" || x == "ChromeOS Flex" || x == "Chrome OS Flex"
            })
        })
        .filter_map(|x| Some((x.title?, x.content?, x.updated?)))
        .map(|(title, content, updated)| {
            let parsed = content
                .body
                .map(|x| {
                    html2md(
                        x.replace("<u>", "<strong>").replace("</u>", "</strong>"),
                        opts.decorator,
                    )
                })
                .unwrap_or_else(|| "No content.".to_string());

            let mut should_filter = !opts.unfiltered;

            let mut lines: Vec<&str> = parsed.split('\n').collect();
            if should_filter {
                lines.dedup();
                lines.truncate(lines.len().saturating_sub(4));
            }

            let mut filtered: Vec<&str> = Vec::new();
            let mut summary = String::new();

            for line in lines.iter() {
                if should_filter {
                    if line.contains("is being updated")
                        || line.contains("has been updated")
                        || line.contains("is updated in")
                        || line.contains("was updated in")
                        || line.contains("has been promoted to")
                        || line.contains("A new LT")
                        || line.contains("The new LT")
                    {
                        let formatted = line
                            .split("Want to know")
                            .next()
                            .map(|x| x.trim())
                            .unwrap_or(line);
                        summary = formatted.into();
                        filtered.push(formatted);
                    } else if line.contains("See the latest release")
                        || line.contains("Release notes for")
                    {
                        filtered.push(line);
                    } else if line.contains("This update contains selective Security fixes")
                        || line.contains("This update contains selected Security fixes")
                        || line.contains("This update contains multiple Security fixes")
                        || line.contains("ChromeOS Vulnerability Bug Fixes")
                        || line.contains("Security Fixes And Rewards")
                    {
                        filtered.push("");
                        filtered.push(line);
                        should_filter = false;
                    }
                } else {
                    filtered.push(line);
                }
            }
            Release {
                title: title.content,
                summary,
                content: filtered.join("\n"),
                timestamp: updated,
            }
        })
        .collect())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Cli::parse();
    let mut releases = get_releases(&opts).await?;
    releases.sort_by(|x, y| y.timestamp.cmp(&x.timestamp));
    let xdg = xdg::BaseDirectories::with_prefix("crosreleasenotifier")?;
    let diff_file = xdg.place_cache_file("last_release")?;
    if opts.diff
        && let Ok(diff_date) = std::fs::read(diff_file.clone()).and_then(|x| {
            serde_json::from_slice::<DateTime<Utc>>(&x).map_err(std::io::Error::other)
        })
    {
        releases.retain(|x| x.timestamp > diff_date)
    }
    match opts.format {
        OutputFormat::Json => {
            serde_json::to_writer(std::io::stdout(), &releases)?;
        }
        OutputFormat::Pretty => {
            if !releases.is_empty() {
                println!(
                    "{}",
                    releases
                        .iter()
                        .map(|x| format!(
                            "============\n{}\nReleased at {}\n============\n{}",
                            x.title,
                            x.timestamp.format("%d/%m/%Y %H:%M"),
                            x.content
                        ))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }
        }
        OutputFormat::Notification => {
            for release in releases.iter() {
                let summary = format!(
                    "ChromeOS Release on {}",
                    release.timestamp.format("%Y/%m/%d")
                );
                notify_rust::Notification::new()
                    .summary(&summary)
                    .body(&release.summary)
                    .timeout(notify_rust::Timeout::Never)
                    .show()?;
            }
        }
    }
    if opts.diff
        && let Some(latest) = releases.first()
    {
        serde_json::to_writer(std::fs::File::create(diff_file)?, &latest.timestamp)?;
    }
    Ok(())
}
