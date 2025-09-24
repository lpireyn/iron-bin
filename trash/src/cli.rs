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

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Perform various operations on the trash.
#[derive(Clone, Debug, Parser, PartialEq)]
#[command(name = "trash", version)]
pub struct Cli {
    /// Command.
    #[command(subcommand)]
    pub command: Command,
}

/// Command.
#[derive(Clone, Debug, PartialEq, Subcommand)]
pub enum Command {
    /// List the files in the trash.
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// Put files in the trash.
    #[command()]
    Put(PutArgs),

    /// Restore files from the trash.
    #[command()]
    Restore(RestoreArgs),

    /// Empty the trash.
    #[command()]
    Empty(EmptyArgs),
}

/// Arguments to the `list` command.
#[derive(Args, Clone, Debug, PartialEq)]
pub struct ListArgs {
    /// Verbose output.
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Print human-readable sizes.
    ///
    /// Useful with the '-v'/'--verbose' option.
    // NOTE: The short name `-h` is more conventional, but it conflicts with the help option
    #[arg(long, short = 'H')]
    pub human_readable: bool,

    /// Sort order.
    // TODO: Use SortOrder::default() as default value
    #[arg(
        default_value = "path",
        long = "sort",
        short = 's',
        value_name = "ORDER"
    )]
    pub sort_order: SortOrder,
}

/// Sort order for the `list` command.
#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub enum SortOrder {
    /// Path, ascending.
    #[default]
    Path,

    /// Deletion time, descending.
    Date,
}

/// Arguments to the `put` command.
#[derive(Args, Clone, Debug, PartialEq)]
pub struct PutArgs {
    /// Prompt before every path.
    #[arg(long, short = 'i')]
    pub interactive: bool,

    /// Verbose output.
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Paths.
    #[arg(required = true, value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// Arguments to the `restore` command.
#[derive(Args, Clone, Debug, PartialEq)]
pub struct RestoreArgs {
    /// Prompt before every path.
    #[arg(long, short = 'i')]
    pub interactive: bool,

    /// Verbose output.
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Paths.
    ///
    /// Defaults to the most recently trashed file.
    ///
    /// Should be quoted to avoid shell expansion.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// Arguments to the `empty` command.
#[derive(Args, Clone, Debug, PartialEq)]
pub struct EmptyArgs {
    /// Do not prompt before emptying the trash.
    #[arg(long, short = 'f')]
    pub force: bool,

    /// Verbose output.
    #[arg(long, short = 'v')]
    pub verbose: bool,
}
