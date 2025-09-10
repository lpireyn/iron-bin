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

mod dir_sizes;
mod info;

use std::{cell::OnceCell, collections::HashMap, os::unix::fs::MetadataExt};

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use chrono::NaiveDateTime;
use dir_sizes::DirSizes;
use info::TrashInfo;
use xdg::BaseDirectories;

use crate::camino_ext::read_dir_utf8_or_empty;

const TRASHINFO_EXTENSION: &str = "trashinfo";

/// Trash.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Trash {
    base_dir: Utf8PathBuf,
    info_dir: Utf8PathBuf,
    files_dir: Utf8PathBuf,
    dir_sizes: OnceCell<DirSizes>,
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
    pub(crate) fn default_base_dir() -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(
            BaseDirectories::default()
                .get_data_home()
                .expect("undefined environment variable: HOME"),
        )
        .expect("XDG data home contains invalid UTF-8 characters")
        .join("Trash")
    }

    /// Create a trash at the given base directory.
    pub(crate) fn new(base_dir: impl Into<Utf8PathBuf>) -> Self {
        let base_dir = base_dir.into();
        let info_dir = base_dir.join("info");
        let files_dir = base_dir.join("files");
        Self {
            base_dir,
            info_dir,
            files_dir,
            dir_sizes: OnceCell::new(),
        }
    }

    /// Return the base directory of this trash.
    pub(crate) fn base_dir(&self) -> &Utf8Path {
        &self.base_dir
    }

    /// Return the info directory of this trash.
    fn info_dir(&self) -> &Utf8Path {
        &self.info_dir
    }

    /// Return the files directory of this trash.
    fn files_dir(&self) -> &Utf8Path {
        &self.files_dir
    }

    /// Return an iterator on the entries of this trash.
    pub(crate) fn entries(&self) -> Result<impl Iterator<Item = Result<TrashEntry>>> {
        let entries = self.trashinfo_paths()?.map(|path| self.new_entry(&path));
        Ok(entries)
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

        let trashinfo_paths = read_dir_utf8_or_empty(self.info_dir())?.filter_map(|dir_entry| {
            // NOTE: If dir_entry cannot be obtained, it is skipped
            let dir_entry = dir_entry.ok()?;
            let path = dir_entry.into_path();
            if is_trashinfo_file(&path) {
                Some(path)
            } else {
                None
            }
        });
        Ok(trashinfo_paths)
    }

    fn new_entry(&self, trashinfo_path: impl AsRef<Utf8Path>) -> Result<TrashEntry> {
        let trashinfo_path = trashinfo_path.as_ref();
        // Determine identifier
        let file_name = trashinfo_path
            .file_name()
            .expect("trashinfo path has no file name");
        // NOTE: Utf8Path has no `base_name` method, so we strip the extension ourselves
        let identifier =
            String::from(&file_name[..file_name.len() - (1 + TRASHINFO_EXTENSION.len())]);
        // Load trashinfo
        let trashinfo = TrashInfo::load_from_file(trashinfo_path)
            .with_context(|| format!("cannot read trashinfo file {trashinfo_path}"))?;
        // Examine file
        let file_path = self.files_dir.join(&identifier);
        let file_metadata = file_path
            .symlink_metadata()
            .with_context(|| format!("cannot get metadata of file {file_path}"))?;
        let size = if file_metadata.is_dir() {
            // The file is a directory
            // Get its size from the cached directory sizes
            let trashinfo_metadata = trashinfo_path.metadata().with_context(|| {
                format!("cannot get metadata of trashinfo file {trashinfo_path}")
            })?;
            let trashinfo_mtime = trashinfo_metadata.mtime() as u64;
            if let Some(dir_size) = self.dir_sizes().get(&identifier)
                && dir_size.mtime() == trashinfo_mtime
            {
                dir_size.size()
            } else {
                // NOTE: We don't compute the actual directory size here
                0
            }
        } else {
            // The file is a regular file or a symlink
            // Get its actual size
            file_metadata.len()
        };
        let entry = TrashEntry {
            identifier,
            original_path: trashinfo.path().to_owned(),
            deletion_time: trashinfo.deletion_time().to_owned(),
            size,
        };
        Ok(entry)
    }

    fn dir_sizes(&self) -> &DirSizes {
        self.dir_sizes.get_or_init(|| {
            self.load_dir_sizes()
                // NOTE: If the directory sizes cannot be loaded, return an empty map
                .unwrap_or_default()
        })
    }

    fn load_dir_sizes(&self) -> Result<HashMap<String, dir_sizes::DirSize>> {
        let path = self.base_dir.join("directorysizes");
        dir_sizes::load_from_file(path)
    }
}

impl Default for Trash {
    /// Create the default home trash.
    fn default() -> Self {
        Self::new(Self::default_base_dir())
    }
}

/// Compute an identifier for the given path.
///
/// # Panics
///
/// This function panics if the given path has no file name.
fn identifier(path: impl AsRef<Utf8Path>) -> String {
    let path = path.as_ref();
    path.file_name().expect("path has no file name").to_string()
}

/// Trash entry.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TrashEntry {
    identifier: String,
    original_path: Utf8PathBuf,
    deletion_time: NaiveDateTime,
    size: u64,
}

impl TrashEntry {
    pub(crate) fn original_path(&self) -> &Utf8Path {
        &self.original_path
    }

    pub(crate) fn deletion_time(&self) -> &NaiveDateTime {
        &self.deletion_time
    }

    pub(crate) fn size(&self) -> u64 {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier() {
        assert_eq!(identifier(Utf8PathBuf::from("/abc/def/ghi.xyz")), "ghi.xyz");
    }
}
