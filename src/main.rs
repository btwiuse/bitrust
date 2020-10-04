extern crate git2;
extern crate serde;
extern crate serde_json;

use std::env::args;
use git2::{Repository, Oid};
use git2::Error as GitError;
use serde::{Serialize, Deserialize};

/// Simple commit, struct determines JSON output
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Commit {
    /// SHA1 hash
    hash: String,
    /// Author name
    author: String,
    /// Commit creationg date as UNIX timestamp
    date: i64,
    /// Commit message
    message: String
}

/// Retrieve commits matching query from repsitory.
fn fetch_commits(repo: &str, rev: &str, query: &str) -> Result<Vec<Commit>, GitError> {
    let repo: Repository = Repository::open(repo)?;
    let rev: Option<Oid> = Oid::from_str(&rev).ok();
    let mut revs = repo.revwalk()?;
    match rev {
        Some(commit_id) => revs.push(commit_id)?,
        _ => revs.push_head()?,
    }

    revs.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;

    let commits = revs
    .filter_map(|commit_id| { repo.find_commit(commit_id.unwrap()).ok() })
    .filter(|commit| { commit.message().map(|m| m.contains(query)).unwrap_or(false) })
    .map(|commit| {
        let c = Commit {
            hash: commit.id().to_string(),
            author: commit.author().name()
                .or(commit.author().email())
                .unwrap_or("Some dude").trim().to_string(),
            date: commit.time().seconds(),
            message: commit.message().unwrap_or("").trim().to_string()
        };
        c
    })
    .collect();

    Ok(commits)
}

fn main(){
    let revision: String = args().nth(1).unwrap_or(String::new());
    let commits = fetch_commits("rust", &revision, "[breaking-change]").unwrap();
    println!("{}", serde_json::to_string_pretty(&commits).unwrap());
}
