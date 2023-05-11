#![allow(unused_imports, dead_code, unused_variables)]
use mdbook::book::Book;
use mdbook::{book::Chapter, renderer::RenderContext, BookItem};
use pulldown_cmark::{BrokenLink, CowStr, Event, Options, Parser, Tag};
use semver::{Version, VersionReq};
extern crate serde;
#[macro_use]
extern crate serde_derive;
use std::fmt::{self, Debug, Formatter};
use std::path::PathBuf;
use std::process;
use std::{error::Error, io};

/// A semver range specifying which versions of `mdbook` this crate supports.
pub const COMPATIBLE_MDBOOK_VERSIONS: &str = "^0.4.0";

fn main() {
    let mut stdin = io::stdin();
    let ctx = RenderContext::from_json(&mut stdin).unwrap();

    version_check(&ctx.version);

    let cfg: UnlinkConfig = ctx
        .config
        .get_deserialized_opt("output.unlink")
        .unwrap_or_default()
        .unwrap_or_default();

    if let Err(broken_links) = check_links(&ctx.book, &cfg) {
        println!("Broken links found:");
        for broken_link in broken_links {
            println!("{:?}", broken_link);
        }
        process::exit(1);
    } else {
        println!("No broken links found")
    }

    // for item in ctx.book.iter() {
    //     if let BookItem::Chapter(ref ch) = *item {
    //         println!("{:#?}", ch);
    //     }
    // }
}

fn check_links<'e>(book: &'e Book, cfg: &UnlinkConfig) -> Result<(), Vec<BrokenLinkError<'e>>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);

    let all_links: Vec<_> = book
        .iter()
        .filter_map(|item| {
            if let BookItem::Chapter(ch) = item {
                if let Some(path) = &ch.path {
                    Some(path)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    println!("All links: {:?}", all_links);

    let mut broken_links = Vec::new();

    for item in book.iter() {
        if let BookItem::Chapter(ref ch) = *item {
            if !cfg.check_drafts && ch.is_draft_chapter() {
                continue;
            }
            // Skip ignored files
            if cfg.include_files.is_empty()
                && cfg.ignore_files.iter().any(|p| {
                    ch.path
                        .as_ref()
                        .unwrap_or(&PathBuf::from(""))
                        .to_str()
                        .unwrap_or_default()
                        .contains(p)
                })
            {
                continue;
            } else if cfg.include_files.iter().any(|p| {
                ch.path
                    .as_ref()
                    .unwrap_or(&PathBuf::from(""))
                    .to_str()
                    .unwrap_or_default()
                    .contains(p)
            }) {
                println!("Checking {:?}", ch.path);
            } else if !cfg.include_files.is_empty() {
                continue;
            }

            let mut broken_link_callback = |broken_link: BrokenLink| {
                println!("BROKEN_LINK: {:?}", broken_link.span);

                // let b_link = BrokenLink {
                //     link_type: broken_link.link_type.clone(),
                //     reference: broken_link.reference.clone(),
                //     span: broken_link.span.clone(),
                // };
                // let malformed_link = BrokenLinkError::MalformedLink(b_link);
                // broken_links.push(malformed_link);
                None
            };
            let parser = Parser::new_with_broken_link_callback(
                &ch.content,
                options,
                Some(&mut broken_link_callback),
            );

            // Iterate through events until start heading
            // Get next until end heading
            let mut in_heading = false;
            let mut current_heading_events = Vec::new();
            let mut heading_ids = Vec::new();
            let heading_parser = Parser::new(&ch.content);
            for event in heading_parser {
                if let Event::Start(tag) = &event {
                    if let Tag::Heading(_, _, _) = tag {
                        in_heading = true;
                    }
                } else if let Event::End(tag) = &event {
                    if let Tag::Heading(_, _, _) = tag {
                        // Construct heading id from current_heading_events
                        let heading_id = current_heading_events
                            .iter()
                            .filter_map(|event| match event {
                                Event::Text(t) => Some(
                                    t.to_string()
                                        .to_ascii_lowercase()
                                        .split(' ')
                                        .collect::<Vec<_>>()
                                        .join("-"),
                                ),
                                Event::Code(t) => Some(
                                    t.to_string()
                                        .to_ascii_lowercase()
                                        .split(' ')
                                        .collect::<Vec<_>>()
                                        .join("-"),
                                ),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join("");
                        heading_ids.push(heading_id);
                        current_heading_events.clear();
                        in_heading = false;
                    }
                }

                if in_heading {
                    current_heading_events.push(event);
                }
            }

            // Reconstruct heading text using

            println!("Heading IDs: {:?}", heading_ids);

            for event in parser {
                println!("{:?}", event);
                let res = match event {
                    Event::Start(ev) => match &ev {
                        Tag::Link(link_type, url, title) | Tag::Image(link_type, url, title) => {
                            println!("1: {:?}", ev);
                            // Check if the link url without the anchor is a valid chapter
                            let mut link_location = url.to_string();
                            println!("2: {:?}", link_location);
                            // Remove the anchor from the link location if one exists, and store in `anchor` variable
                            let anchor = if let Some(anchor_index) = link_location.find('#') {
                                let anchor = link_location.split_off(anchor_index).replace('#', "");
                                Some(anchor)
                            } else {
                                None
                            };
                            println!("3: {:?}", anchor);
                            // Check if the link location is a valid chapter
                            if let Some(path) = &ch.path {
                                println!("4: {:?}", path);
                                let current_absolute_path =
                                    PathBuf::from("./src/").join(link_location);
                                println!("5: {:?}", current_absolute_path);
                                let r = match current_absolute_path.canonicalize() {
                                    Ok(p) => {
                                        // Check if the anchor is valid within the linked chapter
                                        println!("6: {:?}", p);
                                        if let Some(anchor) = anchor {
                                            println!("7: {:?}", anchor);
                                            if heading_ids.contains(&anchor) {
                                                println!("8: {:?}", anchor);
                                                None
                                            } else {
                                                let broken_link =
                                                    BrokenLinkError::NonExistentHeading {
                                                        link_location: p
                                                            .to_str()
                                                            .unwrap_or_default()
                                                            .to_string(),
                                                        link: ev,
                                                    };
                                                println!("9: {:?}", broken_link);
                                                Some(broken_link)
                                            }
                                        } else {
                                            println!("10:");
                                            None
                                        }
                                    }
                                    Err(e) => {
                                        let broken_link = BrokenLinkError::NonExistentChapter {
                                            link_location: current_absolute_path
                                                .to_str()
                                                .unwrap_or_default()
                                                .to_string(),
                                            link: ev,
                                        };
                                        println!("11: {:?}", e);
                                        Some(broken_link)
                                    }
                                };
                                println!("12: {:?}", r);
                            }
                            println!("13:");
                            // All links valid
                            None
                        }
                        _ => {
                            continue;
                        }
                    },
                    _ => {
                        continue;
                    }
                };
                if let Some(broken_link) = res {
                    broken_links.push(broken_link);
                }
            }
        }
    }

    if broken_links.is_empty() {
        Ok(())
    } else {
        Err(broken_links)
    }
}

enum BrokenLinkError<'e> {
    NonExistentChapter {
        link_location: String,
        link: Tag<'e>,
    },
    NonExistentHeading {
        link_location: String,
        link: Tag<'e>,
    },
    MalformedLink(BrokenLink<'e>),
}

impl Debug for BrokenLinkError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BrokenLinkError::NonExistentChapter {
                link_location,
                link,
            } => {
                write!(f, "Non existent chapter: {} ({:?})", link_location, link)
            }
            BrokenLinkError::NonExistentHeading {
                link_location,
                link,
            } => {
                write!(f, "Non existent heading: {} ({:?})", link_location, link)
            }
            BrokenLinkError::MalformedLink(broken_link) => {
                write!(
                    f,
                    "Malformed link: {} [{:?}]",
                    broken_link.reference, broken_link.span
                )
            }
        }
    }
}

fn version_check(version: &str) {
    let requirement = VersionReq::parse(COMPATIBLE_MDBOOK_VERSIONS).unwrap();
    let version = Version::parse(version).expect("mdBook provided an invalid semver version");

    assert!(
        requirement.matches(&version),
        "This version of mdbook-unlink is not compatible with your version of mdBook."
    );
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
struct UnlinkConfig {
    /// A list of glob patterns to ignore files
    pub ignore_files: Vec<String>,
    /// A list of glob patterns to ignore when checking links
    pub ignore_links: Vec<String>,
    /// Whether or not to check draft chapters
    /// Default: true
    pub check_drafts: bool,
    /// A list of files to include when checking links
    pub include_files: Vec<String>,
}
