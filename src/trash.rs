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

//! Trash.

use camino::{Utf8Path, Utf8PathBuf};
use chrono::NaiveDateTime;
use eyre::{ContextCompat, Result, WrapErr};
use ini::Ini;
use xdg::BaseDirectories;

use crate::camino_ext::read_dir_utf8_or_empty;

const TRASHINFO_EXTENSION: &str = "trashinfo";

/// Trash.
#[derive(Clone, Debug, PartialEq)]
pub struct Trash {
    base_dir: Utf8PathBuf,
    info_dir: Utf8PathBuf,
    files_dir: Utf8PathBuf,
}

impl Trash {
    /// Return the default base directory of the home trash.
    ///
    /// # Specification
    ///
    /// The home trash base directory is `$XDG_DATA_HOME/Trash`.
    ///
    /// # Panics
    ///
    /// This function panics if the `HOME` environment variable is not defined or if the XDG data home contains invalid UTF-8 characters.
    pub fn default_base_dir() -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(
            BaseDirectories::default()
                .get_data_home()
                .expect("Undefined environment variable: HOME"),
        )
        .expect("XDG data home contains invalid UTF-8 characters")
        .join("Trash")
    }

    /// Create a trash at the given base directory.
    pub fn new(base_dir: impl Into<Utf8PathBuf>) -> Self {
        let base_dir = base_dir.into();
        let info_dir = base_dir.join("info");
        let files_dir = base_dir.join("files");
        Self {
            base_dir,
            info_dir,
            files_dir,
        }
    }

    /// Compute the identifier for the given path.
    ///
    /// # Panics
    ///
    /// This function panics if the given path has no file name
    fn identifier(path: impl AsRef<Utf8Path>) -> String {
        let path = path.as_ref();
        path.file_name().expect("Path has no file name").to_string()
    }

    /// Return the base directory of this trash.
    pub fn base_dir(&self) -> &Utf8Path {
        self.base_dir.as_path()
    }

    /// Return the info directory of this trash.
    fn info_dir(&self) -> &Utf8Path {
        self.info_dir.as_path()
    }

    /// Return the files directory of this trash.
    fn files_dir(&self) -> &Utf8Path {
        self.files_dir.as_path()
    }

    /// Return an iterator on the trash info files in this trash.
    ///
    /// # Implementation
    ///
    /// This function uses [read_dir_utf8_or_empty] in order to be able to return the same type of iterator,
    /// whether the directory exists or not.
    fn trashinfo_paths(&self) -> Result<impl Iterator<Item = Utf8PathBuf>> {
        fn is_trashinfo_file(path: impl AsRef<Utf8Path>) -> bool {
            let path = path.as_ref();
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == TRASHINFO_EXTENSION)
        }

        let info_paths = read_dir_utf8_or_empty(self.info_dir())?.filter_map(|dir_entry| {
            // NOTE: If dir_entry cannot be obtained, it is skipped
            let dir_entry = dir_entry.ok()?;
            let path = dir_entry.into_path();
            if is_trashinfo_file(&path) {
                Some(path)
            } else {
                None
            }
        });
        Ok(info_paths)
    }

    fn trashinfos(&self) -> Result<impl Iterator<Item = Result<TrashInfo>>> {
        Ok(self.trashinfo_paths()?.map(TrashInfo::load_from_file))
    }

    fn new_entry(&self, trashinfo: &TrashInfo) -> Result<TrashEntry> {
        let identifier = &trashinfo.identifier;
        let trash_file_path = self.files_dir.join(identifier);
        let metadata = trash_file_path
            .symlink_metadata()
            .wrap_err_with(|| format!("Cannot get metadata for file {trash_file_path}"))?;
        let size = if metadata.is_dir() {
            // TODO: Compute size and use `directorysizes` file
            0
        } else {
            metadata.len()
        };
        let entry = TrashEntry {
            identifier: identifier.to_owned(),
            original_path: trashinfo.path.to_owned(),
            deletion_date: trashinfo.deletion_date.to_owned(),
            size,
        };
        Ok(entry)
    }

    /// Return an interator on the entries of this trash.
    pub fn entries(&self) -> Result<impl Iterator<Item = Result<TrashEntry>>> {
        let entries = self
            .trashinfos()?
            .map(|trashinfo| trashinfo.and_then(|trashinfo| self.new_entry(&trashinfo)));
        Ok(entries)
    }
}

impl Default for Trash {
    /// Create the default home trash.
    fn default() -> Self {
        Self::new(Self::default_base_dir())
    }
}

/// Trash info.
///
/// Represents the contents of a `.trashinfo` file in the info directory of a trash.
#[derive(Clone, Debug, PartialEq)]
struct TrashInfo {
    identifier: String,
    path: Utf8PathBuf,
    deletion_date: NaiveDateTime,
}

impl TrashInfo {
    /// Create a trash info from the contents of a `.trashinfo` file.
    ///
    /// # Implementation
    ///
    /// This function uses the rust-ini crate to read the `.trashinfo` file.
    fn load_from_file(path: impl AsRef<Utf8Path>) -> Result<Self> {
        const TRASH_INFO: &str = "Trash Info";
        const PATH: &str = "Path";
        const DELETION_DATE: &str = "DeletionDate";

        let path = path.as_ref();
        let file_name = path.file_name().expect("Trash info path has no file name");
        // NOTE: Utf8Path has no `base_name` method, so we strip the extension ourselves
        let identifier =
            String::from(&file_name[..file_name.len() - (1 + TRASHINFO_EXTENSION.len())]);
        // Ini
        let ini = Ini::load_from_file(path)
            .wrap_err_with(|| format!("Error in trash info file {path}"))?;
        // Section: Trash Info
        let section = ini
            .section(Some(TRASH_INFO))
            .wrap_err_with(|| format!("Trash info file {path} has no `{TRASH_INFO}` section"))?;
        // NOTE
        // The spec says:
        // > If a string that starts with “Path=” or “DeletionDate=” occurs several times, the first occurrence is to be used.
        // TODO: Check if this behavior can be implemented with `Ini`
        // Entry: Path
        let path_entry = section
            .get(PATH)
            .wrap_err_with(|| format!("Trash info file {path} has no `{PATH}` entry"))?;
        let path_entry = urlencoding::decode(path_entry)
            .wrap_err_with(|| format!("Trash info file {path} has invalid URL encoded path"))?;
        // Entry: Deletion date
        let deletion_date_entry = section
            .get(DELETION_DATE)
            .wrap_err_with(|| format!("Trash info file {path} has no `{DELETION_DATE}` entry"))?;
        let deletion_date = deletion_date_entry
            .parse::<NaiveDateTime>()
            .wrap_err_with(|| {
                format!(
                    "Trash info file {path} has invalid deletion date \"{deletion_date_entry}\""
                )
            })?;
        // Trash info
        Ok(Self {
            identifier,
            path: path_entry.as_ref().into(),
            deletion_date,
        })
    }
}

/// Trash entry.
#[derive(Clone, Debug, PartialEq)]
pub struct TrashEntry {
    identifier: String,
    original_path: Utf8PathBuf,
    deletion_date: NaiveDateTime,
    size: u64,
}

impl TrashEntry {
    pub fn original_path(&self) -> &Utf8Path {
        self.original_path.as_path()
    }

    pub fn deletion_date(&self) -> &NaiveDateTime {
        &self.deletion_date
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier() {
        assert_eq!(
            Trash::identifier(Utf8PathBuf::from("/abc/def/ghi.xyz")),
            "ghi.xyz"
        );
    }
}
