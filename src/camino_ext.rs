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

//! Extensions for Camino.

use camino::{ReadDirUtf8, Utf8Path};

/// Wrapper around an optional [ReadDirUtf8].
pub struct OptionalReadDirUtf8(Option<ReadDirUtf8>);

impl Iterator for OptionalReadDirUtf8 {
    type Item = <ReadDirUtf8 as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iter) = &mut self.0 {
            iter.next()
        } else {
            None
        }
    }
}

pub fn read_dir_utf8_or_empty(path: impl AsRef<Utf8Path>) -> std::io::Result<OptionalReadDirUtf8> {
    let path = path.as_ref();
    Ok(OptionalReadDirUtf8(if path.exists() {
        Some(path.read_dir_utf8()?)
    } else {
        None
    }))
}
