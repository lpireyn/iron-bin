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

//! CLI.

use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};
use eyre::Result;
use tabled::{
    Table, Tabled,
    settings::{Alignment, Style, object::Columns},
};

use crate::trash::{Trash, TrashEntry};

/// Perform various operations on the trash.
#[derive(Clone, Debug, Parser)]
#[command(version)]
pub struct Cli {
    /// Command.
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn run() -> Result<()> {
        let cli = Self::parse();
        match &cli.command {
            Command::List(args) => cli.list(args),
        }
    }

    fn list(&self, args: &ListArgs) -> Result<()> {
        let trash = Trash::default();
        let mut entries = trash
            .entries()?
            // NOTE: Errors are discarded
            .filter_map(|entry| entry.ok())
            .collect::<Vec<_>>();
        // Sort trash entries
        let sort_order = &args.sort_order;
        entries.sort_by(|entry1, entry2| sort_order.cmp(entry1, entry2));
        // Print trash entries
        // TODO: Quote paths under certain conditions (see https://www.gnu.org/software/coreutils/quotes.html)
        if !args.verbose {
            for entry in entries {
                println!("{}", entry.original_path())
            }
        } else {
            let mut table = Table::new(entries.iter().map(Record::from));
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

/// Command.
#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// List the files in the trash.
    #[command(visible_alias = "ls")]
    List(ListArgs),
}

/// Arguments to the list command.
#[derive(Args, Clone, Debug)]
struct ListArgs {
    /// Verbose output.
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Sort order.
    // TODO: Use SortOrder::default() as default value
    #[arg(
        default_value = "path",
        long = "sort",
        short = 's',
        value_name = "ORDER"
    )]
    sort_order: SortOrder,
}

/// Sort order for the list command.
#[derive(Clone, Debug, Default, ValueEnum)]
enum SortOrder {
    /// Path, ascending.
    #[default]
    Path,

    /// Deletion date, descending.
    Date,
}

impl SortOrder {
    fn cmp(&self, entry1: &TrashEntry, entry2: &TrashEntry) -> std::cmp::Ordering {
        match *self {
            SortOrder::Path => entry1.original_path().cmp(entry2.original_path()),
            SortOrder::Date => entry2.deletion_date().cmp(entry1.deletion_date()),
        }
    }
}

/// Table record for a trash entry.
#[derive(Tabled)]
struct Record {
    #[tabled(rename = "size")]
    size: u64,

    #[tabled(rename = "deletion date")]
    deletion_date: String,

    #[tabled(rename = "original path")]
    path: Utf8PathBuf,
}

impl From<&TrashEntry> for Record {
    fn from(value: &TrashEntry) -> Self {
        Self {
            size: value.size(),
            deletion_date: value.deletion_date().format("%c").to_string(),
            path: value.original_path().to_owned(),
        }
    }
}
