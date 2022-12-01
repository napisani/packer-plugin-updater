use crate::result::Result;
use git2::{Direction, FetchOptions, Oid, Repository};
use std::{fs, path};
#[derive(Debug)]
pub enum RemoteHeadType {
    Head,
    Pull,
    Tag,
    Unknown { type_name: String },
}
impl RemoteHeadType {
    pub fn from_str(type_str: &str) -> Self {
        match type_str {
            "heads" => Self::Head,
            "pulls" => Self::Pull,
            "tags" => Self::Tag,
            _ => Self::Unknown {
                type_name: type_str.to_owned(),
            },
        }
    }
}

#[derive(Debug)]
pub struct RemoteHeadCommit {
    pub sha: String,
    pub name: String,
    pub message: String,
    pub full_name: String,
    pub type_: RemoteHeadType,
}
impl RemoteHeadCommit {
    pub fn from_current_commit(sha: String) -> Self {
        Self {
            sha,
            name: "(Current)".to_owned(),
            full_name: "(Current)".to_owned(),
            type_: RemoteHeadType::Unknown {
                type_name: "Current".to_owned(),
            },
            message: "".to_owned(),
        }
    }
    pub fn from_remote_ls(sha: String, full_name: String, message: String) -> Self {
        let full_name_orig = full_name.clone();
        let ref_parts: Vec<&str> = full_name.split('/').collect();
        if ref_parts.len() >= 3 {
            let name = &ref_parts[2..].join("/");
            return Self {
                sha,
                name: name.to_owned(),
                full_name: full_name_orig,
                type_: RemoteHeadType::from_str(ref_parts[1]),
                message,
            };
        }

        Self {
            sha,
            name: full_name_orig.clone(),
            full_name: full_name_orig,
            type_: RemoteHeadType::Head,
            message,
        }
    }
}
impl ToString for RemoteHeadCommit {
    fn to_string(&self) -> String {
        format!("({}) {}", &self.sha[..6], self.name)
    }
}
pub fn find_latest_commits(repo: &Repository) -> Result<Vec<RemoteHeadCommit>> {
    let mut remote = repo
        .find_remote("origin")
        .or_else(|_| repo.remote_anonymous("origin"))?;

    // fetch remote 
    let mut fo = FetchOptions::new();
    remote.download(&[] as &[&str], Some(&mut fo))?;

    // Connect to the remote and call the printing function for each of the
    // remote references.
    let connection = remote.connect_auth(Direction::Fetch, None, None)?;

    Ok(connection
        .list()?
        .iter()
        .map(|head| {
            RemoteHeadCommit::from_remote_ls(
                head.oid().to_string(),
                head.name().to_string(),
                get_commit_message(repo, &head.oid().to_string()),
            )
        })
        .collect())
}
fn get_commit_message(repo: &Repository, sha: &str) -> String {
    match repo.find_commit(Oid::from_str(sha).unwrap()) {
        Ok(commit) => commit.message().unwrap_or("no commit message").to_owned(),
        _ => "commit not found".to_owned(),
    }
}
pub fn get_repo(p: &path::Path) -> Result<Repository> {
    let repo = Repository::open(p.display().to_string())?;
    Ok(repo)
}
pub fn get_remote_branch_name(p: &path::Path) -> Result<String> {
    let remote_head_ref_file_path = p
        .join(".git")
        .join("refs")
        .join("remotes")
        .join("origin")
        .join("HEAD");
    let full_ref = fs::read_to_string(&remote_head_ref_file_path)?;
    let re = regex::Regex::new(r"ref:\s*refs/remotes/origin/").unwrap();
    let full_ref = re.replace_all(full_ref.trim(), "").to_string();
    Ok(full_ref)
}

