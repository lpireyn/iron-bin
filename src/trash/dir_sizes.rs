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

//! Directory sizes.

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Context, Result, anyhow};
use camino::Utf8Path;

/// Directory size.
///
/// Represents a record in the `directorysizes` file of a trash.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct DirSize {
    name: String,
    size: u64,
    mtime: u64,
}

impl DirSize {
    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn size(&self) -> u64 {
        self.size
    }

    pub(super) fn mtime(&self) -> u64 {
        self.mtime
    }

    fn load_from_line(line: impl AsRef<str>) -> Result<DirSize> {
        let line = line.as_ref();
        let mut iter = line.split_whitespace();
        let size = iter.next().ok_or_else(|| anyhow!("missing size"))?;
        let size = size
            .parse::<u64>()
            .with_context(|| format!("invalid size: {size}"))?;
        let mtime = iter.next().ok_or_else(|| anyhow!("missing mtime"))?;
        let mut mtime = mtime
            .parse::<u64>()
            .with_context(|| format!("invalid mtime: {mtime}"))?;
        // NOTE
        // The spec says:
        // > The modification time is stored as an integer, the number of seconds since Epoch.
        // However, some implementations (e.g. Dolphin) use timestamps in *milliseconds* since Epoch.
        // So we assume timestamps beyond the year 2100 are given in milliseconds, and correct accordingly.
        if mtime > 4_200_000_000 {
            mtime /= 1000;
        }
        let name = iter.next().ok_or_else(|| anyhow!("missing name"))?;
        let name = urlencoding::decode(name)
            .with_context(|| format!("invalid name: {name}"))?
            .into_owned();
        // NOTE: Additional fields, if any, are ignored
        let dir_size = DirSize { name, size, mtime };
        Ok(dir_size)
    }
}

/// Return the given timestamp corrected.
///
/// # Details
///
/// The spec says:
///
/// > The modification time is stored as an integer, the number of seconds since Epoch.
///
/// So it is assumed the spec mandates timestamps in seconds since Epoch.
/// However, some implementations (e.g. Dolphin) use timestamps in *milliseconds* since Epoch.
/// Therefore, timestamps after 2200-01-01 are assumed to be in milliseconds and are corrected accordingly.
fn corrected_timestamp(timestamp: u64) -> u64 {
    /// Roughly 2200-01-01 at midnight
    const LIMIT: u64 = 7_258_122_000;

    if timestamp > LIMIT {
        timestamp / 1000
    } else {
        timestamp
    }
}

pub(super) type DirSizes = HashMap<String, DirSize>;

pub(super) fn load_from_file(path: impl AsRef<Utf8Path>) -> Result<DirSizes> {
    let path = path.as_ref();
    let mut dir_sizes = DirSizes::new();
    // Return an empty map if the file doesn't exist (or is not a file)
    if path.is_file() {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if let Ok(dir_size) = DirSize::load_from_line(line) {
                dir_sizes.insert(dir_size.name.clone(), dir_size);
            }
        }
    }
    Ok(dir_sizes)
}
