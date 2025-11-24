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

//! Simple y/n prompt.

use std::io;

use anyhow::{Context, Result};

/// Prompt the user for a y/n answer to a question.
///
/// Any answer that starts with 'y' or 'Y' is treated as a yes;
/// anything else (including an empty answer) is treated as a no.
pub fn prompt(question: impl AsRef<str>) -> Result<bool> {
    let question = question.as_ref();
    eprint!("{question} [y/N] ");
    let mut answer = String::with_capacity(10);
    io::stdin()
        .read_line(&mut answer)
        .context("cannot prompt")?;
    Ok(answer.to_lowercase().starts_with('y'))
}
