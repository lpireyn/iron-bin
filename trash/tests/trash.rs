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

//! Integration tests for the `trash` command.

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::{NamedTempFile, TempDir};
use predicates::prelude::*;
use std::ffi::OsStr;
use std::fs::create_dir_all;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn temp_dir() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    Ok(temp_dir)
}

fn temp_file(name: impl AsRef<OsStr>, contents: impl AsRef<str>) -> Result<NamedTempFile> {
    let temp_file = NamedTempFile::new(name)?;
    temp_file.write_str(contents.as_ref())?;
    Ok(temp_file)
}

fn trash_command(data_dir: impl AsRef<Path>) -> Result<Command> {
    let data_dir = data_dir.as_ref();
    let mut command = Command::cargo_bin("trash")?;
    command.env("XDG_DATA_HOME", data_dir);
    Ok(command)
}

#[test]
fn test_option_version() -> Result<()> {
    let pkg_version = env!("CARGO_PKG_VERSION");
    trash_command(temp_dir()?.path())?
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("trash {pkg_version}")))
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_option_help() -> Result<()> {
    trash_command(temp_dir()?.path())?
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--version"))
        .stdout(predicate::str::contains("--help"))
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_list_absent_trash() -> Result<()> {
    let data_dir = temp_dir()?;
    let mut command = trash_command(data_dir.path())?;
    command
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
    assert_eq!(
        data_dir.read_dir()?.count(),
        0,
        "the trash directory was created"
    );
    Ok(())
}

#[test]
fn test_list_empty_trash() -> Result<()> {
    let data_dir = temp_dir()?;
    for dir in ["info", "files"] {
        create_dir_all(data_dir.join("Trash").join(dir))?;
    }
    trash_command(data_dir.path())?
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_put_file_and_list() -> Result<()> {
    let data_dir = temp_dir()?;
    let file = temp_file("test.txt", "abc")?;
    trash_command(data_dir.path())?
        .arg("put")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
    assert!(!file.exists(), "the trashed file is still present");
    trash_command(data_dir.path())?
        .arg("list")
        .assert()
        .success()
        .stdout(format!("{}\n", file.path().to_str().unwrap()))
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_put_files() -> Result<()> {
    let data_dir = temp_dir()?;
    let file1 = temp_file("test1.txt", "abc")?;
    let file2 = temp_file("test2.txt", "def")?;
    trash_command(data_dir.path())?
        .arg("put")
        .arg(file1.path())
        .arg(file2.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
    assert!(!file1.exists(), "the trashed file is still present");
    assert!(!file2.exists(), "the trashed file is still present");
    Ok(())
}

#[test]
fn test_put_and_restore_file() -> Result<()> {
    let data_dir = temp_dir()?;
    let file1 = temp_file("test1.txt", "abc")?;
    let file2 = temp_file("test2.txt", "def")?;
    trash_command(data_dir.path())?
        .arg("put")
        .arg(file1.path())
        .arg(file2.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
    trash_command(data_dir.path())?
        .arg("restore")
        .arg(file1.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
    assert!(file1.exists(), "the restored file is absent");
    assert!(!file2.exists(), "the file not restored is present");
    Ok(())
}

#[test]
fn test_put_file_and_restore_most_recent() -> Result<()> {
    let temp_dir = temp_dir()?;
    let file = temp_file("test.txt", "abc")?;
    trash_command(temp_dir.path())?
        .arg("put")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
    trash_command(temp_dir.path())?
        .arg("restore")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
    assert!(file.exists(), "the restored file is absent");
    Ok(())
}
