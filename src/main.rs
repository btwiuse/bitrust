extern crate git2;
extern crate serde;
extern crate serde_json;

use git2::{Oid, Repository, Revwalk, Error, Sort};
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
    message: String,
}

/// Retrieve commits matching query from repsitory.
fn fetch_commits(repo: &str, rev: &str, query: &str) -> Result<Vec<Commit>, Error> {
    let repo: Repository = Repository::open(repo)?;
    let mut walker: Revwalk = repo.revwalk()?;

    // At least one commit must be pushed onto the walker before a walk can be started.
    match Oid::from_str(rev).ok() {
        Some(commit_id) => walker.push(commit_id)?,
        _ => walker.push_head()?,
    }

    walker.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;

    let commits = walker.filter_map(|commit_id| repo.find_commit(commit_id.unwrap()).ok())
        .filter(|commit| commit.message().map(|m| m.contains(query)).unwrap_or(false))
        .map(|commit| {
            Commit {
                hash: commit.id().to_string(),
                author: commit.author()
                    .name()
                    .or(commit.author().email())
                    .unwrap_or("Some dude")
                    .trim()
                    .to_string(),
                date: commit.time().seconds(),
                message: commit.message().unwrap_or("").trim().to_string(),
            }
        })
        .collect();

    Ok(commits)
}

fn main() {
    let revision = std::env::args().nth(1).unwrap_or(String::new());
    let commits = fetch_commits("./rust", revision.trim(), "[breaking-change]").unwrap();
    println!("{}", serde_json::to_string_pretty(&commits).unwrap());
}
