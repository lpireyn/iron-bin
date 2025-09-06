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

//! Cargo build.

#[path = "src/cli.rs"]
mod cli;

use std::{
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use clap::{Command, CommandFactory};
use clap_mangen::Man;
use cli::Cli;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    // Generate man pages
    println!("cargo:rerun-if-changed=src/cli.rs");
    let out_dir =
        PathBuf::from(env::var_os("OUT_DIR").expect("Undefined environment variable: OUT_DIR"));
    let mut cli = Cli::command();
    cli.build();
    gen_man_page(&out_dir, &cli)
}

fn gen_man_page(dir: &Path, command: &Command) -> Result<()> {
    let name = command
        .get_display_name()
        // The root command has no display name, so use its name instead
        .unwrap_or_else(|| command.get_name());
    let mut file = File::create(dir.join(format!("{name}.1")))?;
    let man = Man::new(command.clone());
    man.render(&mut file)?;
    file.flush()?;
    // Don't generate man pages for subcommands of the `help` command
    // FIXME: The subcommands of the `help` command still appear in the root man page
    if command.get_name() != "help" {
        for subcommand in command.get_subcommands() {
            gen_man_page(dir, subcommand)?;
        }
    }
    Ok(())
}
