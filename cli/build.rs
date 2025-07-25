// Copyright 2023 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::Path;
use std::process::Command;
use std::str;

const GIT_HEAD_PATH: &str = "../.git/HEAD";
const JJ_OP_HEADS_PATH: &str = "../.jj/repo/op_heads/heads";

fn main() {
    let version = std::env::var("CARGO_PKG_VERSION").unwrap();

    if Path::new(GIT_HEAD_PATH).exists() {
        // In colocated repo, .git/HEAD should reflect the working-copy parent.
        println!("cargo:rerun-if-changed={GIT_HEAD_PATH}");
    } else if Path::new(JJ_OP_HEADS_PATH).exists() {
        // op_heads changes when working-copy files are mutated, which is way more
        // frequent than .git/HEAD.
        println!("cargo:rerun-if-changed={JJ_OP_HEADS_PATH}");
    }
    println!("cargo:rerun-if-env-changed=NIX_JJ_GIT_HASH");

    if let Some(git_hash) = get_git_hash() {
        println!("cargo:rustc-env=JJ_VERSION={version}-{git_hash}");
    } else {
        println!("cargo:rustc-env=JJ_VERSION={version}");
    }

    let docs_symlink_path = Path::new("docs");
    println!("cargo:rerun-if-changed={}", docs_symlink_path.display());
    if docs_symlink_path.join("index.md").exists() {
        println!("cargo:rustc-env=JJ_DOCS_DIR=docs/");
    } else {
        println!("cargo:rustc-env=JJ_DOCS_DIR=../docs/");
    }
}

fn get_git_hash() -> Option<String> {
    if let Some(nix_hash) = std::env::var("NIX_JJ_GIT_HASH")
        .ok()
        .filter(|s| !s.is_empty())
    {
        return Some(nix_hash);
    }
    if let Ok(output) = Command::new("jj")
        .args([
            "--ignore-working-copy",
            "--color=never",
            "log",
            "--no-graph",
            "-r=@-",
            "-T=commit_id ++ '-'",
        ])
        .output()
    {
        if output.status.success() {
            let mut parent_commits = String::from_utf8(output.stdout).unwrap();
            // If a development version of `jj` is compiled at a merge commit, this will
            // result in several commit ids separated by `-`s.
            parent_commits.truncate(parent_commits.trim_end_matches('-').len());
            return Some(parent_commits);
        }
    }

    if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if output.status.success() {
            let line = str::from_utf8(&output.stdout).unwrap();
            return Some(line.trim_end().to_owned());
        }
    }

    None
}
