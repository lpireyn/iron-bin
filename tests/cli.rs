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

use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn trash_command() -> Result<Command> {
    Ok(Command::cargo_bin("trash")?)
}

#[test]
fn option_version() -> Result<()> {
    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");
    trash_command()?
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "{pkg_name} {pkg_version}"
        )))
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn option_help() -> Result<()> {
    trash_command()?
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--version"))
        .stdout(predicate::str::contains("--help"))
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn absent_trash_dir() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    trash_command()?
        .arg("list")
        .env("XDG_DATA_HOME", temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
    // Check that the trash directory was not created
    assert_eq!(temp_dir.read_dir()?.count(), 0);
    Ok(())
}
