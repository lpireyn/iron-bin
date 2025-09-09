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
    io::{IsTerminal, stdout},
};

use anyhow::Result;
use camino::Utf8Path;
use clap::Parser;
use fast_glob::glob_match;
use humansize::{DECIMAL, FormatSizeOptions, make_format};
use shell_quote::Sh;
use tabled::{
    Table, Tabled,
    settings::{Alignment, Style, object::Columns},
};

use crate::{
    cli::{Cli, Command, ListArgs, SortOrder},
    trash::{Trash, TrashEntry},
};

/// Application.
#[derive(Clone, Debug)]
pub struct App;

impl App {
    pub fn run() -> Result<()> {
        let cli = Cli::parse();
        let app = App;
        match &cli.command {
            Command::List(args) => app.list(args),
        }
    }

    fn list(&self, args: &ListArgs) -> Result<()> {
        // NOTE: This doesn't behave exactly as the quoting in `ls` but it's safe enough
        fn quoted(path: impl AsRef<Utf8Path>, quote: bool) -> String {
            let path = path.as_ref();
            if quote {
                str::from_utf8(&Sh::quote_vec(path.as_str()))
                    .unwrap()
                    .to_string()
            } else {
                path.to_string()
            }
        }

        let trash = Trash::default();
        let patterns = &args.patterns;
        let mut entries = trash
            .entries()?
            // NOTE: Errors are discarded
            .filter_map(|entry| entry.ok())
            // Filter entries according to patterns, if any
            .filter(|entry| {
                patterns.is_empty()
                    || patterns
                        .iter()
                        .any(|pattern| glob_match(pattern, entry.original_path().as_str()))
            })
            .collect::<Vec<_>>();
        // Sort entries
        let compare: fn(&TrashEntry, &TrashEntry) -> Ordering = match &args.sort_order {
            SortOrder::Path => |entry1, entry2| entry1.original_path().cmp(entry2.original_path()),
            SortOrder::Date => |entry1, entry2| entry2.deletion_time().cmp(entry1.deletion_time()),
        };
        entries.sort_by(compare);
        // Print entries
        // NOTE: Quote paths only if stdout is a terminal
        let is_terminal = stdout().is_terminal();
        if !args.verbose {
            for entry in entries {
                println!("{}", quoted(entry.original_path(), is_terminal))
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
                deletion_time: entry.deletion_time().format("%c").to_string(),
                path: quoted(entry.original_path(), is_terminal),
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
}

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
