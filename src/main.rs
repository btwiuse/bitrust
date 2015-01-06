extern crate git2;
extern crate "rustc-serialize" as rustc_serialize;

use std::io::File;
use std::fmt::Show;
use std::os;
use git2::{Repository, Oid};
use rustc_serialize::json;
use rustc_serialize::hex::ToHex;

#[derive(RustcDecodable, RustcEncodable)]
struct Commit {
    hash: String,
    author: String,
    date: i64,
    message: String
}

fn main() {
    let mut output = File::create(&os::getcwd().unwrap().join("log.json")).unwrap();

    let repo_path = os::getcwd().unwrap().join("rust");
    let repo = match Repository::open(&repo_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to init `{}`: {}", repo_path.display(), e),
    };
    let mut revs = match repo.revwalk() {
        Ok(revs) => revs,
        Err(e) => panic!("failed to revwalk: {}", e),
    };

    revs.push_head().unwrap();

    let commits = revs
    .filter_map(|commitId| { repo.find_commit(commitId).ok() })
    .filter_map(|commit| {
        match commit.message().and_then(|msg| { Some(msg.contains("[breaking-change]")) }) {
            Some(true) => Some(commit),
            _ => None
        }
    })
    .map(|commit| {
        Commit {
            hash: commit.id().as_bytes().to_hex().to_string(),
            author: commit.author().name()
                .or(commit.author().email())
                .or(Some("Some dude"))
                .unwrap().trim().to_string(),
            date: commit.time().seconds(),
            message: commit.message()
                .or(Some(""))
                .unwrap().trim().to_string()
        }
    })
    .take(100)
    .collect::<Vec<Commit>>();

    write!(&mut output, "{}", json::as_pretty_json(&commits));
}
