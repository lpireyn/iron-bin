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
//!
//! # Implementation
//!
//! The `rust-ini` crate is used to read and write the `.trashinfo` files.

use std::io;

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use chrono::NaiveDateTime;
use ini::Ini;

const SECTION_TRASH_INFO: &str = "Trash Info";
const ENTRY_PATH: &str = "Path";
const ENTRY_DELETION_DATE: &str = "DeletionDate";

/// Trash info.
///
/// Represents the contents of a `.trashinfo` file in the info directory of a trash.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct TrashInfo {
    pub(super) path: Utf8PathBuf,
    pub(super) deletion_time: NaiveDateTime,
}

impl TrashInfo {
    /// Read a trash info from the given reader.
    pub(super) fn read_from(reader: &mut impl io::Read) -> Result<Self> {
        let ini = Ini::read_from(reader)?;
        TrashInfo::try_from(&ini)
    }

    /// Write this trash info to the given writer.
    pub(super) fn write_to(&self, writer: &mut impl io::Write) -> Result<()> {
        Ini::from(self).write_to(writer)?;
        Ok(())
    }
}

impl TryFrom<&Ini> for TrashInfo {
    type Error = anyhow::Error;

    fn try_from(ini: &Ini) -> std::result::Result<Self, Self::Error> {
        // Section: Trash Info
        let section = ini
            .section(Some(SECTION_TRASH_INFO))
            .with_context(|| format!("missing section: {SECTION_TRASH_INFO}"))?;
        // NOTE
        // The spec says:
        // > If a string that starts with “Path=” or “DeletionDate=” occurs several times, the first occurrence is to be used.
        // TODO: Check if this behavior can be implemented with `Ini`
        // Entry: Path
        let path_entry = section
            .get(ENTRY_PATH)
            .with_context(|| format!("missing entry: {ENTRY_PATH}"))?;
        let path_entry = urlencoding::decode(path_entry)
            .with_context(|| format!("invalid path: {path_entry}"))?;
        let path_entry = path_entry.as_ref();
        // Entry: Deletion date
        let deletion_date_entry = section
            .get(ENTRY_DELETION_DATE)
            .with_context(|| format!("missing entry: {ENTRY_DELETION_DATE}"))?;
        let deletion_date = deletion_date_entry
            .parse::<NaiveDateTime>()
            .with_context(|| format!("invalid deletion date: {deletion_date_entry}"))?;
        // Trash info
        let info = Self {
            path: path_entry.into(),
            deletion_time: deletion_date,
        };
        Ok(info)
    }
}

impl From<&TrashInfo> for Ini {
    fn from(info: &TrashInfo) -> Ini {
        let mut ini = Ini::new();
        ini
            // Section: Trash Info
            .with_section(Some(SECTION_TRASH_INFO))
            // Entry: Path
            .set(ENTRY_PATH, urlencoding::encode(info.path.as_str()))
            // Entry: Deletion date
            .set(
                ENTRY_DELETION_DATE,
                info.deletion_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
            );
        ini
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};

    use super::*;

    #[test]
    fn test_read_from() {
        let mut trashinfo: &[u8] = b"\
[Trash Info]
Path=%2Fabc%2Fdef%2Fghi.xyz
DeletionDate=2025-02-17T13:14:15
        ";
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
        let mut bytes = Vec::<u8>::new();
        trashinfo.write_to(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            b"\
[Trash Info]
Path=%2Fabc%2Fdef%2Fghi.xyz
DeletionDate=2025-02-17T13:14:15
"
        );
    }
}
