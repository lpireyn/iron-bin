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

use std::{
    cell::OnceCell,
    fs::{File, OpenOptions, create_dir_all, remove_dir_all, remove_file, rename},
    io::{BufReader, BufWriter, ErrorKind},
    os::unix::fs::MetadataExt,
};

use anyhow::{Context, Result, bail};
use camino::{Utf8Path, Utf8PathBuf};
use chrono::{Local, NaiveDateTime};
use dir_sizes::DirSizes;
use info::TrashInfo;
use xdg::BaseDirectories;

use crate::camino_ext::read_dir_utf8_or_empty;

const TRASHINFO_EXTENSION: &str = "trashinfo";

/// Trash.
#[derive(Clone, Debug, PartialEq)]
pub struct Trash {
    base_dir: Utf8PathBuf,
    info_dir: Utf8PathBuf,
    files_dir: Utf8PathBuf,
    directorysizes_file: Utf8PathBuf,
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
    pub fn default_base_dir() -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(
            BaseDirectories::default()
                .get_data_home()
                .expect("undefined environment variable: HOME"),
        )
        .expect("XDG data home contains invalid UTF-8 characters")
        .join("Trash")
    }

    /// Create a trash at the given base directory.
    pub fn new(base_dir: impl Into<Utf8PathBuf>) -> Self {
        let base_dir = base_dir.into();
        let info_dir = base_dir.join("info");
        let files_dir = base_dir.join("files");
        let directorysizes_file = base_dir.join("directorysizes");
        Self {
            base_dir,
            info_dir,
            files_dir,
            directorysizes_file,
            dir_sizes: OnceCell::new(),
        }
    }

    /// Return the base directory of this trash.
    pub fn base_dir(&self) -> &Utf8Path {
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

    /// Return the directorysizes file of this trash.
    fn directorysizes_file(&self) -> &Utf8Path {
        &self.directorysizes_file
    }

    /// Return an iterator on the entries of this trash.
    pub fn entries(&self) -> Result<impl Iterator<Item = Result<TrashEntry>>> {
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
        let trashinfo_file = File::open(trashinfo_path)?;
        let trashinfo = TrashInfo::read_from(&mut BufReader::new(trashinfo_file))?;
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

    fn create_dirs(&self) -> Result<()> {
        for dir in [&self.base_dir, &self.info_dir, &self.files_dir] {
            create_dir_all(dir)
                .with_context(|| format!("cannot create trash directory at {dir}"))?;
        }
        Ok(())
    }

    pub fn put(&self, path: impl AsRef<Utf8Path>) -> Result<TrashPutReport> {
        let path = path.as_ref().canonicalize_utf8()?;
        let deletion_time = Local::now().naive_local();
        let trashinfo = TrashInfo::new(&path, deletion_time);
        self.create_dirs()?;
        let (identifier, trashinfo_file) = self.open_new_trashinfo_file(&path)?;
        trashinfo.write_to(&mut BufWriter::new(trashinfo_file))?;
        let file_path = self.files_dir.join(identifier);
        rename(&path, &file_path)?;
        let report = TrashPutReport {
            path,
            deletion_time,
        };
        Ok(report)
    }

    /// Create and open a new `.trashinfo` file in this trash for the given path.
    fn open_new_trashinfo_file(&self, path: impl AsRef<Utf8Path>) -> Result<(String, File)> {
        let path = path.as_ref();
        let base_identifier = identifier(path);
        let mut number = 0_u16;
        loop {
            let identifier = if number == 0 {
                base_identifier.clone()
            } else {
                format!("{base_identifier}_{number}")
            };
            let trashinfo_path = self
                .info_dir
                .join(format!("{identifier}.{TRASHINFO_EXTENSION}"));
            match OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(trashinfo_path)
            {
                // The trashinfo file could be created
                Ok(file) => break Ok((identifier, file)),
                // The trashinfo file could not be created
                Err(err) => match err.kind() {
                    // A trashinfo file already exists for the identifier
                    ErrorKind::AlreadyExists => {
                        // TODO: Handle overflow
                        number += 1;
                        continue;
                    }
                    // Another error occurred
                    _ => {
                        break Err(err)
                            .with_context(|| format!("cannot create trashinfo file for {path}"));
                    }
                },
            }
        }
    }

    pub fn restore(&self, identifier: impl AsRef<str>) -> Result<TrashRestoreReport> {
        let identifier = identifier.as_ref();
        // Read trashinfo
        let trashinfo_path = self
            .info_dir
            .join(format!("{identifier}.{TRASHINFO_EXTENSION}"));
        let trashinfo = {
            let mut trashinfo_file = File::open(&trashinfo_path)
                .with_context(|| format!("cannot open trashinfo file {trashinfo_path}"))?;
            TrashInfo::read_from(&mut trashinfo_file)
                .with_context(|| format!("cannot read trashinfo file {trashinfo_path}"))?
        };
        let original_path = trashinfo.path();
        let deletion_time = trashinfo.deletion_time();
        // Check if original path is available
        if original_path.exists() {
            bail!("file {original_path} already exists");
        }
        // Determine trash file
        let file_path = self.files_dir.join(identifier);
        if !file_path.exists() {
            bail!("file {file_path} not found");
        }
        // Move trash file to original path
        rename(&file_path, original_path)
            .with_context(|| format!("cannot move file {file_path} to {original_path}"))?;
        // Remove trashinfo file
        remove_file(&trashinfo_path)
            .with_context(|| format!("cannot remove trashinfo file {trashinfo_path}"))?;
        // TODO: If file is a directory, remove it from dir sizes
        let report = TrashRestoreReport {
            path: original_path.to_owned(),
            deletion_time: deletion_time.to_owned(),
        };
        Ok(report)
    }

    pub fn empty(&self) -> Result<TrashEmptyReport> {
        let mut entry_count = 0_usize;
        // Remove trashinfo files
        for trashinfo_path in self.trashinfo_paths()? {
            remove_file(&trashinfo_path)
                .with_context(|| format!("cannot remove trashinfo file {trashinfo_path}"))?;
        }
        // Remove trash files
        for dir_entry in self.files_dir.read_dir_utf8()? {
            let dir_entry = dir_entry.context("cannot read contents of trash files directory")?;
            let file_path = dir_entry.path();
            remove_dir_all(file_path).with_context(|| format!("cannot remove file {file_path}"))?;
            entry_count += 1;
        }
        // Remove dir sizes file
        let directorysizes_file = self.directorysizes_file();
        remove_file(directorysizes_file)
            .with_context(|| format!("cannot remove directorysizes file {directorysizes_file}"))?;
        let report = TrashEmptyReport {
            entry_count,
            // TODO: Compute size
            size: 0,
        };
        Ok(report)
    }

    fn dir_sizes(&self) -> &DirSizes {
        self.dir_sizes.get_or_init(|| {
            self.load_dir_sizes()
                // NOTE: If the directory sizes cannot be loaded, return an empty map
                .unwrap_or_default()
        })
    }

    fn load_dir_sizes(&self) -> Result<DirSizes> {
        let path = self.directorysizes_file();
        let mut file = File::open(path)?;
        dir_sizes::read_from(&mut file)
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
pub struct TrashEntry {
    identifier: String,
    original_path: Utf8PathBuf,
    deletion_time: NaiveDateTime,
    size: u64,
}

impl TrashEntry {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn original_path(&self) -> &Utf8Path {
        &self.original_path
    }

    pub fn deletion_time(&self) -> &NaiveDateTime {
        &self.deletion_time
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrashPutReport {
    pub path: Utf8PathBuf,
    pub deletion_time: NaiveDateTime,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrashRestoreReport {
    pub path: Utf8PathBuf,
    pub deletion_time: NaiveDateTime,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrashEmptyReport {
    pub entry_count: usize,
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use assert_fs::{NamedTempFile, TempDir, prelude::FileWriteStr};

    use super::*;

    fn new_test_trash() -> Trash {
        let base_dir = TempDir::new().unwrap();
        let base_dir = Utf8Path::from_path(base_dir.path()).unwrap();
        Trash::new(base_dir)
    }

    #[test]
    fn test_identifier() {
        let identifier = identifier(Utf8PathBuf::from("/abc/def/ghi.xyz"));
        assert_eq!(identifier, "ghi.xyz");
    }

    #[test]
    fn test_entries_empty() {
        let trash = new_test_trash();
        let entries = trash.entries().unwrap().collect::<Vec<_>>();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_put_path_not_found() {
        let trash = new_test_trash();
        let test_dir = TempDir::new().unwrap();
        let test_file = Utf8Path::from_path(test_dir.path())
            .unwrap()
            .join("test.txt");
        let result = trash.put(&test_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_put_file() {
        let trash = new_test_trash();
        let test_file = NamedTempFile::new("test").unwrap();
        test_file.write_str("abc").unwrap();
        let test_file_path = Utf8Path::from_path(test_file.path()).unwrap();
        let test_file_canonical_path = test_file_path.canonicalize_utf8().unwrap();
        let test_file_size = test_file.symlink_metadata().unwrap().len();
        trash.put(test_file_path).unwrap();
        let entries = trash.entries().unwrap().collect::<Vec<_>>();
        assert_eq!(entries.len(), 1);
        let entry = entries.first().unwrap().as_ref().unwrap();
        assert_eq!(entry.original_path, test_file_canonical_path);
        assert_eq!(entry.size, test_file_size);
    }
}
