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
    io::{BufRead, BufReader, Read},
};

use anyhow::{Context, Result, anyhow};

/// Directory size.
///
/// Represents a record in the `directorysizes` file of a trash.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct DirSize {
    pub(super) name: String,
    pub(super) size: u64,
    pub(super) mtime: u64,
}

pub(super) type DirSizes = HashMap<String, DirSize>;

impl DirSize {
    fn read_from_line(line: impl AsRef<str>) -> Result<DirSize> {
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

        let line = line.as_ref();
        let mut iter = line.split_whitespace();
        let size = iter.next().ok_or_else(|| anyhow!("missing size"))?;
        let size = size
            .parse::<u64>()
            .with_context(|| format!("invalid size: {size}"))?;
        let mtime = iter.next().ok_or_else(|| anyhow!("missing mtime"))?;
        let mtime = mtime
            .parse::<u64>()
            .with_context(|| format!("invalid mtime: {mtime}"))?;
        let mtime = corrected_timestamp(mtime);
        let name = iter.next().ok_or_else(|| anyhow!("missing name"))?;
        let name = urlencoding::decode(name)
            .with_context(|| format!("invalid name: {name}"))?
            .into_owned();
        // NOTE: Additional fields, if any, are ignored
        let dir_size = DirSize { name, size, mtime };
        Ok(dir_size)
    }
}

pub(super) fn read_from(reader: &mut impl Read) -> Result<DirSizes> {
    let mut dir_sizes = DirSizes::new();
    let reader = BufReader::new(reader);
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        // NOTE: Invalid lines are ignored
        if let Ok(dir_size) = DirSize::read_from_line(line) {
            dir_sizes.insert(dir_size.name.clone(), dir_size);
        }
    }
    Ok(dir_sizes)
}
