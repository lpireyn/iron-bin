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

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Perform various operations on the trash.
#[derive(Clone, Debug, Parser, PartialEq)]
#[command(name = "trash", version)]
pub(crate) struct Cli {
    /// Command.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// Command.
#[derive(Clone, Debug, PartialEq, Subcommand)]
pub(crate) enum Command {
    /// List the files in the trash.
    #[command(visible_alias = "ls")]
    List(ListArgs),
}

/// Arguments to the list command.
#[derive(Args, Clone, Debug, PartialEq)]
pub(crate) struct ListArgs {
    /// Verbose output.
    #[arg(long, short = 'v')]
    pub(crate) verbose: bool,

    /// Print human-readable sizes.
    ///
    /// Useful with the '-v'/'--verbose' option.
    // NOTE: The short name `-h` is more conventional, but it conflicts with the help option
    #[arg(long, short = 'H')]
    pub(crate) human_readable: bool,

    /// Sort order.
    // TODO: Use SortOrder::default() as default value
    #[arg(
        default_value = "path",
        long = "sort",
        short = 's',
        value_name = "ORDER"
    )]
    pub(crate) sort_order: SortOrder,

    /// Path patterns.
    ///
    /// Should be quoted to avoid shell expansion.
    // TODO: Document supported patterns (see https://docs.rs/fast-glob/latest/fast_glob/#syntax)
    #[arg(value_name = "PATTERN")]
    pub(crate) patterns: Vec<String>,
}

/// Sort order for the list command.
#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub(crate) enum SortOrder {
    /// Path, ascending.
    #[default]
    Path,

    /// Deletion time, descending.
    Date,
}
