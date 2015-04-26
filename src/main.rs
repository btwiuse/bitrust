extern crate git2;
extern crate rustc_serialize;

use std::io::{stdout, Write};
use std::env;
use git2::{Repository, Oid};
use git2::Error as GitError;
use rustc_serialize::json;

/// Simple commit, struct determines JSON output
#[derive(Clone, RustcDecodable, RustcEncodable)]
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
fn fetch_commits(repo: &Repository, start: &Option<Oid>, query: &str, amount: usize)
    -> Result<Vec<Commit>, GitError> {

    let mut revs = try!(repo.revwalk());
    match *start {
        Some(commit_id) => try!(revs.push(commit_id)),
        _ => try!(revs.push_head()),
    }

    revs.set_sorting(git2::SORT_TOPOLOGICAL | git2::SORT_TIME);

    let commits = revs
    .filter_map(|commit_id| { repo.find_commit(commit_id).ok() })
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
    .take(amount)
    .collect();

    Ok(commits)
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let start = if args.len() >= 2 { Some(Oid::from_str(&args[1][..]).unwrap()) } else { None };

    let cwd = env::current_dir().unwrap();
    let repo = Repository::open(&cwd.join("rust")).unwrap();

    let commits = fetch_commits(&repo, &start, "[breaking-change]", 100).unwrap();

    write!(&mut stdout(), "{}\n", json::as_pretty_json(&commits)).unwrap();
}
