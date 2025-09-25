// Copyright 2025 Laurent Pireyn
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Application.

use std::{
    cmp::Ordering,
    fmt::Display,
    io::{self, IsTerminal, stdout},
};

use anyhow::{Context, Result, bail};
use camino::Utf8Path;
use chrono::NaiveDateTime;
use clap::Parser;
use humansize::{DECIMAL, FormatSizeOptions, make_format};
use iron_bin::{Trash, TrashEmptyReport, TrashEntry};
use shell_quote::Sh;
use tabled::{
    Table, Tabled,
    settings::{Alignment, Style, object::Columns},
};

use crate::cli::{Cli, Command, EmptyArgs, ListArgs, PutArgs, RestoreArgs, SortOrder};

/// Application.
#[derive(Clone, Debug)]
pub struct App;

/// Table record for a trash entry.
#[derive(Tabled)]
struct Record {
    #[tabled(rename = "size")]
    size: String,

    #[tabled(rename = "deletion time")]
    deletion_time: String,

    #[tabled(rename = "original path")]
    path: String,
}

impl App {
    pub fn run() -> Result<()> {
        let cli = Cli::parse();
        let app = App;
        match &cli.command {
            Command::List(args) => app.list(args),
            Command::Put(args) => app.put(args),
            Command::Restore(args) => app.restore(args),
            Command::Empty(args) => app.empty(args),
        }
    }

    fn list(&self, args: &ListArgs) -> Result<()> {
        fn not_quoted(path: impl AsRef<Utf8Path>) -> String {
            let path = path.as_ref();
            path.to_string()
        }

        // NOTE: This doesn't behave exactly as the quoting in `ls` but it's safe enough
        fn quoted(path: impl AsRef<Utf8Path>) -> String {
            let path = path.as_ref();
            str::from_utf8(&Sh::quote_vec(path.as_str()))
                .unwrap()
                .to_string()
        }

        let trash = Trash::default();
        let entries = trash.entries()?;
        // Discard entries in error
        let mut entries = entries.filter_map(|entry| entry.ok()).collect::<Vec<_>>();
        // Sort entries according to sort order
        entries.sort_by(comparator(&args.sort_order));
        // Print entries
        let should_quote = stdout().is_terminal();
        let maybe_quoted = if should_quote { quoted } else { not_quoted };
        if !args.verbose {
            for entry in &entries {
                println!("{}", maybe_quoted(entry.original_path()))
            }
        } else {
            // NOTE: We use the DECIMAL format but remove the space after the value to mimic the behavior of `ls -lh`
            let size_formatter =
                make_format(FormatSizeOptions::from(DECIMAL).space_after_value(false));
            let mut table = Table::new(entries.iter().map(|entry| Record {
                size: if args.human_readable {
                    size_formatter(entry.size())
                } else {
                    format!("{}", entry.size())
                },
                deletion_time: format_datetime(entry.deletion_time()).to_string(),
                path: maybe_quoted(entry.original_path()),
            }));
            table
                .with(Style::empty())
                .modify(Columns::first(), Alignment::right());
            // TODO: Hide column headers
            println!("total {}", entries.len());
            println!("{table}");
        }
        Ok(())
    }

    fn put(&self, args: &PutArgs) -> Result<()> {
        let trash = Trash::default();
        let PutArgs {
            paths,
            interactive,
            verbose,
        } = args;
        let should_prompt = *interactive && stdout().is_terminal();
        let mut trashed = 0_usize;
        let mut errors = 0_usize;
        // Discard invalid UTF-8 paths
        let paths = paths
            .iter()
            .filter_map(|path| {
                Utf8Path::from_path(path).or_else(|| {
                    eprintln!("invalid UTF-8 path: {}", path.display());
                    errors += 1;
                    None
                })
            })
            .collect::<Vec<_>>();
        for path in paths {
            if !should_prompt || prompt(format!("trash {path}?"))? {
                match trash.put(path) {
                    Ok(report) => {
                        if *verbose {
                            println!(
                                "trashed {} on {}",
                                report.path,
                                format_datetime(&report.deletion_time)
                            );
                        }
                        trashed += 1;
                    }
                    Err(err) => {
                        eprintln!("cannot trash {path}: {err:#}");
                        errors += 1;
                    }
                }
            }
        }
        if *verbose {
            println!("total {trashed} trashed");
        }
        if errors > 0 {
            bail!("{errors} not trashed");
        }
        Ok(())
    }

    fn restore(&self, args: &RestoreArgs) -> Result<()> {
        let trash = Trash::default();
        let RestoreArgs {
            interactive,
            verbose,
            paths,
        } = args;
        // Make paths absolute
        // NOTE: We cannot use PathBuf::canonicalize here, as the paths likely don't exist anymore
        let current_dir = std::env::current_dir().context("cannot determine current directory")?;
        let paths = paths
            .iter()
            .map(|path| current_dir.join(path))
            .collect::<Vec<_>>();
        let mut restored = 0_usize;
        let mut errors = 0_usize;
        // Discard invalid UTF-8 paths
        let paths = paths
            .iter()
            .filter_map(|path| {
                Utf8Path::from_path(path).or_else(|| {
                    eprintln!("invalid UTF-8 path: {}", path.display());
                    errors += 1;
                    None
                })
            })
            .collect::<Vec<_>>();
        // Get entries
        let entries = trash.entries()?;
        // Discard entries in error
        let mut entries = entries.filter_map(|entry| entry.ok()).collect::<Vec<_>>();
        // Sort entries by deletion time, descending
        entries.sort_by(comparator(&SortOrder::Date));
        // Determine entries to restore
        let entries = if paths.is_empty() {
            // No paths specified, take the most recent entry
            if let Some(entry) = entries.first() {
                vec![entry]
            } else {
                bail!("empty trash");
            }
        } else {
            // Path(s) specified, take most recent entry for each path
            paths
                .into_iter()
                .filter_map(|path| {
                    entries
                        .iter()
                        .find(|entry| entry.original_path() == path)
                        .or_else(|| {
                            eprintln!("file {path} not found in trash");
                            None
                        })
                })
                .collect()
        };
        // Restore entries
        let should_prompt = *interactive && stdout().is_terminal();
        for entry in entries {
            let identifier = entry.identifier();
            let original_path = entry.original_path();
            let deletion_time = entry.deletion_time();
            let deletion_time_disp = format_datetime(deletion_time);
            if !should_prompt
                || prompt(format!(
                    "restore {original_path} trashed on {deletion_time_disp}?"
                ))?
            {
                match trash.restore(identifier) {
                    Ok(report) => {
                        if *verbose {
                            println!(
                                "restored {} trashed on {}",
                                report.path, report.deletion_time
                            );
                        }
                        restored += 1;
                    }
                    Err(err) => {
                        eprintln!("cannot restore {}: {err:#}", entry.original_path());
                        errors += 1;
                    }
                }
            }
        }
        if *verbose {
            println!("total {restored} restored");
        }
        if errors > 0 {
            bail!("{errors} not restored");
        }
        Ok(())
    }

    fn empty(&self, args: &EmptyArgs) -> Result<()> {
        let trash = Trash::default();
        let EmptyArgs { force, verbose } = args;
        let should_prompt = !*force && stdout().is_terminal();
        if !should_prompt || prompt("empty trash?")? {
            let TrashEmptyReport {
                entry_count,
                size: _,
            } = trash.empty()?;
            if *verbose {
                // TODO: Print the size
                println!("total {entry_count} removed");
            }
        }
        Ok(())
    }
}

fn comparator(sort_order: &SortOrder) -> fn(&TrashEntry, &TrashEntry) -> Ordering {
    match sort_order {
        SortOrder::Path => |entry1, entry2| entry1.original_path().cmp(entry2.original_path()),
        SortOrder::Date => |entry1, entry2| entry2.deletion_time().cmp(entry1.deletion_time()),
    }
}

fn format_datetime(datetime: &NaiveDateTime) -> impl Display {
    datetime.format("%c")
}

/// Prompt the user for a y/n answer to a question.
fn prompt(question: impl AsRef<str>) -> Result<bool> {
    let question = question.as_ref();
    eprint!("{question} [y/N] ");
    let mut answer = String::with_capacity(10);
    io::stdin()
        .read_line(&mut answer)
        .context("cannot prompt")?;
    Ok(answer.to_lowercase().starts_with('y'))
}
