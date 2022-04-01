#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

//! This crate gathers relevant git info from any Repo. Some of the info returned includes:
//! - **Git status info**: Checks if a repo is dirty, has been modified and so on.
//! - **Commits**: Gathers and shows information for the last 10 commits
//! 
//! ## Example
//! ```rust
//!  # let mut path = env::current_dir().unwrap();
//!  # path.push("test_project");
//!  # let dir = path.to_string_lossy().to_string();
//!  // let dir = "/path/to/repo"; <- Point to the location of t=your repo
//!  let info = Info::new(&dir).status_info()?.commit_info()?;
//!  println("{:#?}", info);

// Copyright 2022 Anthony Mugendi
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

use anyhow::Result;
use chrono::{DateTime, Utc};
use cmd_lib::run_fun;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};
use std::{collections::HashMap, path::PathBuf};

/// The Status Struct:
/// Holds information about the status of the repo
#[derive(Debug, Clone)]
pub struct Status {
    /// Holds any error thrown by ```git status```
    pub error: Option<String>,
    /// Indicates if repo is dirty or not. For this, we check both ```git status -s``` and ```git diff -stat```
    pub git_dirty: Option<bool>,
    /// A HashMap describing the state of the repo
    pub summary: HashMap<String, bool>,
}

/// Struct holding info of each commit
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Commit {
    /// The repo commit date
    #[serde(with = "my_date_format")]
    pub commit_date: Option<DateTime<Utc>>,
    /// The repo commit message
    pub commit_message: Option<String>,
    /// The repo author name
    pub author_name: Option<String>,
    /// The repo author email
    pub author_email: Option<String>,
    /// The repo committer name (sometimes the author is not always the committer)
    pub committer_name: Option<String>,
    /// The repo committer email
    pub committer_email: Option<String>,
    /// tree hash
    pub tree_hash: Option<String>,
}

/// The main struct that returns combined Status and Commits info
#[derive(Debug, Clone)]
pub struct Info {
    /// Repo directory
    pub dir: String,
    /// Boolean indicating id the directory above is indeed a repo
    pub is_git: bool,
    /// Repo branch inspected
    pub branch: Option<String>,
    /// Status information for the repo
    pub status: Option<Status>,
    /// Information on the repo commits
    pub commits: Option<Vec<Commit>>,
}


impl Commit {
    /// To initialize a blank Commit Struct
    pub fn new() -> Commit {
        Commit {
            // branch: "".into(),
            commit_date: None,
            commit_message: None,
            author_name: None,
            author_email: None,
            committer_name: None,
            committer_email: None,
            tree_hash: None,
        }
    }
}

impl Info {
    /// To initialize the Info Struct. A &str pointing to the repo directory is expected
    /// This implementation method checks that the directory does indeed exist and that the repo is a git repo
    /// It returns a new Info Struct with the "dir" and "is_git" fields set
    /// 
    /// ## Example
    /// ```
    ///  # let mut path = env::current_dir().unwrap();
    ///  # path.push("test_project");
    ///  # let dir = path.to_string_lossy().to_string();
    ///  // let dir = "/path/to/repo"; <- Point to the location of t=your repo
    ///  let info = Info::new(&dir);
    ///  println("{:#?}", info);
    /// ```
    pub fn new(dir: &str) -> Info {
        // check if dir is_git
        let mut project_path = PathBuf::from(dir);
        project_path.push(".git");

        let is_git = project_path.exists();

        Info {
            dir: dir.into(),
            is_git: is_git,
            status: None,
            commits: None,
            branch: None,
        }
    }

    /// Get information of all the commits.
    /// This Method returns Info in its result.
    /// If there are no commits, the returned value is None
    /// ## Example
    /// ```
    ///  # let mut path = env::current_dir().unwrap();
    ///  # path.push("test_project");
    ///  # let dir = path.to_string_lossy().to_string();
    ///  // let dir = "/path/to/repo"; <- Point to the location of t=your repo
    ///  let commits_info = Info::new(&dir).commit_info()?;
    ///  println("{:#?}", commits_info);
    /// ```
    pub fn commit_info(&self) -> Result<Info> {
        let mut git_info = self.clone();

        if git_info.is_git {
            let dir = &git_info.dir;

            //check diff
            let branch = match run_fun!(
                cd ${dir};
                git branch -r |  grep -v HEAD | head -n 1 ;
            ) {
                Ok(resp) => {
                    let r = resp.clone();
                    r
                }
                _ => "".into(),
            };

            let branch = &branch[..];
            let branch = branch.trim();
            // println!("BBB >> {:?}", branch);
            git_info.branch = Some(branch.into());

            let format = format!("{{\"commit_date\":\"%ci\", \"commit_message\":\"%s\", \"author_name\":\"%an\", \"author_email\":\"%ae\", \"committer_name\":\"%cn\", \"committer_email\":\"%ce\",  \"tree_hash\":\"%t\"}}");

            // let format = "%ci";

            let empty_commit = json!(Commit::new());

            let commits = match run_fun!(
                cd ${dir};
                git log --format="$format" $branch
                // git status
            ) {
                Ok(resp) => resp,
                Err(_) => {
                    // println!("{:#?}", e);
                    // Commit::new()
                    empty_commit.to_string()
                }
            };

            // println!("{:#?}", commits);

            let commits = commits.split("\n").collect::<Vec<&str>>();
            let len: usize = if commits.len() > 5 { 5 } else { commits.len() };

            // pick top
            let top_commits: Vec<Commit> = commits[0..len]
                .to_vec()
                .iter()
                .map(|s| {
                    let commit: Commit = match from_str(s) {
                        Ok(c) => c,
                        _ => Commit::new(),
                    };
                    commit
                })
                .filter(|e: &Commit| {
                    // let b:&Commit = e;
                    e.commit_date != None
                })
                .collect();

            git_info.commits = if top_commits.len() > 0 {
                Some(top_commits)
            } else {
                None
            };

            // println!("{:#?}",);
            // git_info
        }
        Ok(git_info)
    }

    /// This method returns status information for the repo
    /// ## Example
    /// ```
    ///  # let mut path = env::current_dir().unwrap();
    ///  # path.push("test_project");
    ///  # let dir = path.to_string_lossy().to_string();
    ///  // let dir = "/path/to/repo"; <- Point to the location of t=your repo
    ///  let status_info = Info::new(&dir).status_info()?;
    ///  println("{:#?}", status_info);
    /// ```
    pub fn status_info(&self) -> Result<Info> {
        let mut git_info = self.clone();
        let mut status = Status {
            error: None,
            git_dirty: None,
            summary: HashMap::new(),
        };

        if git_info.is_git {
            let dir = &git_info.dir;

            match run_fun!( cd ${dir}; git status -s; ) {
                // if we can run git status then it is a git directory
                Ok(resp) => {
                    //
                    let is_modified = resp.len() > 0;

                    //check diff
                    let resp = match run_fun!( cd ${dir}; git diff --stat; ) {
                        Ok(r) => r,
                        _ => "ERR".into(),
                    };
                    let is_dirty = resp.len() > 0;

                    status.summary.insert("is_modified".into(), is_modified);
                    status.summary.insert("is_dirty".into(), is_dirty);
                    status.git_dirty = Some(is_dirty || is_modified);
                }
                Err(e) => {
                    status.error = Some(format!("{:?}", e));
                }
            };
        }

        git_info.status = Some(status);

        Ok(git_info)
    }
}

mod my_date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    // 2014-08-29 16:09:40 -0600

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S %Z";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match date {
            Some(dt) => {
                let s = format!("{}", dt.format(FORMAT));
                s
            }
            _ => "null".into(),
        };

        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let dt = Utc
            .datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)?;

        Ok(Some(dt))
    }
}


// To successfully run tests, first create a "test_project" directory at the home of this crate
// Do so by running cargo new test_project
// It is not included so you will need to create it yourself
#[cfg(test)]
mod tests {

    use super::Info;
    use std::env;

    fn test_dir() -> String {
        let mut path = env::current_dir().unwrap();
        path.push("test_project");

        path.to_string_lossy().to_string()
    }

    #[test]
    fn it_works() {
        let dir = test_dir();

        let info = Info::new(&dir)
            .status_info()
            .expect("Unable to get status info")
            .commit_info()
            .expect("Unable to get commit info");

        assert_eq!(None, info.commits);
        assert_eq!(Some(true), info.status.expect("err").git_dirty);
    }
}
