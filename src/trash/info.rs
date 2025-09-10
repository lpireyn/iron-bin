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

//! Trash info.

use std::io::{Read, Write};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use chrono::NaiveDateTime;
use ini::Ini;

const TRASH_INFO: &str = "Trash Info";
const PATH: &str = "Path";
const DELETION_DATE: &str = "DeletionDate";

/// Trash info.
///
/// Represents the contents of a `.trashinfo` file in the info directory of a trash.
///
/// # Implementation
///
/// The rust-ini crate is used to read and write the trash info.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct TrashInfo {
    path: Utf8PathBuf,
    deletion_time: NaiveDateTime,
}

impl TrashInfo {
    pub(super) fn new(
        path: impl Into<Utf8PathBuf>,
        deletion_time: impl Into<NaiveDateTime>,
    ) -> Self {
        Self {
            path: path.into(),
            deletion_time: deletion_time.into(),
        }
    }

    /// Read trash info from the given reader.
    pub(super) fn read_from(reader: &mut impl Read) -> Result<Self> {
        // Ini
        let ini = Ini::read_from(reader)?;
        // Section: Trash Info
        let section = ini
            .section(Some(TRASH_INFO))
            .with_context(|| format!("missing section: {TRASH_INFO}"))?;
        // NOTE
        // The spec says:
        // > If a string that starts with “Path=” or “DeletionDate=” occurs several times, the first occurrence is to be used.
        // TODO: Check if this behavior can be implemented with `Ini`
        // Entry: Path
        let path_entry = section
            .get(PATH)
            .with_context(|| format!("missing entry: {PATH}"))?;
        let path_entry = urlencoding::decode(path_entry)
            .with_context(|| format!("invalid path: {path_entry}"))?;
        // Entry: Deletion date
        let deletion_date_entry = section
            .get(DELETION_DATE)
            .with_context(|| format!("missing entry: {DELETION_DATE}"))?;
        let deletion_date = deletion_date_entry
            .parse::<NaiveDateTime>()
            .with_context(|| format!("invalid deletion date: {deletion_date_entry}"))?;
        // Trash info
        let trashinfo = Self {
            path: path_entry.as_ref().into(),
            deletion_time: deletion_date,
        };
        Ok(trashinfo)
    }

    pub(super) fn path(&self) -> &Utf8Path {
        &self.path
    }

    pub(super) fn deletion_time(&self) -> &NaiveDateTime {
        &self.deletion_time
    }

    fn to_ini(&self) -> Ini {
        let mut ini = Ini::new();
        ini
            // Section: Trash Info
            .with_section(Some(TRASH_INFO))
            // Entry: Path
            .set(PATH, urlencoding::encode(self.path.as_str()))
            // Entry: Deletion date
            .set(
                DELETION_DATE,
                self.deletion_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            );
        ini
    }

    /// Write this trash info to the given writer.
    pub(super) fn write_to(&self, writer: &mut impl Write) -> Result<()> {
        self.to_ini().write_to(writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};

    use super::*;

    #[test]
    fn test_read_from() {
        let mut trashinfo = "[Trash Info]
Path=%2Fabc%2Fdef%2Fghi.xyz
DeletionDate=2025-02-17T13:14:15
"
        .as_bytes();
        let trashinfo = TrashInfo::read_from(&mut trashinfo).unwrap();
        assert_eq!(
            trashinfo,
            TrashInfo {
                path: Utf8PathBuf::from("/abc/def/ghi.xyz"),
                deletion_time: NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2025, 2, 17).unwrap(),
                    NaiveTime::from_hms_opt(13, 14, 15).unwrap(),
                ),
            }
        );
    }

    #[test]
    fn test_write_to() {
        let trashinfo = TrashInfo {
            path: Utf8PathBuf::from("/abc/def/ghi.xyz"),
            deletion_time: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2025, 2, 17).unwrap(),
                NaiveTime::from_hms_opt(13, 14, 15).unwrap(),
            ),
        };
        let mut v = Vec::<u8>::new();
        trashinfo.write_to(&mut v).unwrap();
        assert_eq!(
            String::from_utf8(v).unwrap(),
            "[Trash Info]
Path=%2Fabc%2Fdef%2Fghi.xyz
DeletionDate=2025-02-17T13:14:15
"
        );
    }
}
