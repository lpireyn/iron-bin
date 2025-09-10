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

//! Prompt.

use anyhow::{Context, Result};
use dialoguer::Confirm;

/// Prompt the user for a y/n answer to a question.
pub(crate) fn prompt(question: impl AsRef<str>) -> Result<bool> {
    let question = question.as_ref();
    Confirm::new()
        .with_prompt(question)
        .wait_for_newline(true)
        .report(false)
        .interact()
        .context("cannot prompt")
}
