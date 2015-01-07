extern crate git2;
extern crate "rustc-serialize" as rustc_serialize;

use std::io::File;
use std::os;
use git2::{Repository, Error};
use rustc_serialize::json;
use rustc_serialize::hex::ToHex;

#[derive(RustcDecodable, RustcEncodable)]
struct Commit {
    hash: String,
    author: String,
    date: i64,
    message: String
}

fn fetch_commits(repo: &Repository, query: &str, amount: uint) -> Result<Vec<Commit>, Error> {
    let mut revs = try!(repo.revwalk());
    try!(revs.push_head());

    let commits = revs
    .filter_map(|commit_id| { repo.find_commit(commit_id).ok() })
    .filter_map(|commit| {
        match commit.message().and_then(|msg| { Some(msg.contains(query)) }) {
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
    .take(amount)
    .collect();

    Result::Ok(commits)
}

fn main() {
    let cwd = os::getcwd().unwrap();
    let repo = Repository::open(&cwd.join("rust")).unwrap();

    let mut output = File::create(&cwd.join("log.json")).unwrap();

    let commits = fetch_commits(&repo, "[breaking-change]", 100).unwrap();

    match write!(&mut output, "{}", json::as_pretty_json(&commits)) {
        Ok(_) => println!("wrote commits to `log.json`."),
        Err(e) => panic!("failed to write commits to file: {}", e),
    };
}
