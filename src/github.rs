use jiff::ToSpan;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{cache::Cache, error::Error, url_fetch::UrlFetch};

#[derive(Serialize, Deserialize)]
pub struct GitHubOwner {
    //pub login: String,
    //pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct GitHubRepoInfo {
    //pub name: String,
    //pub full_name: String,
    //pub private: bool,
    //pub owner: GitHubOwner,
    //pub fork: bool,
    pub default_branch: String,
}

#[derive(Serialize, Deserialize)]
pub struct GitHubBranchInfo {
    //pub name: String,
    pub commit: GitHubCommit,
}

#[derive(Serialize, Deserialize)]
pub struct GitHubCommit {
    pub sha: String,
    pub commit: GitHubCommitInfo,
}

#[derive(Serialize, Deserialize)]
pub struct GitHubCommitInfo {
    pub author: GitHubCommitAuthor,
    //pub committer: GitHubCommitAuthor,
}

#[derive(Serialize, Deserialize)]
pub struct GitHubCommitAuthor {
    //pub name: String,
    //pub email: String,
    pub date: jiff::Timestamp,
}

pub struct GitHubRepo {
    base_url: String,
    base_key: String,
    cache: Cache,
}

impl GitHubRepo {
    pub fn new(owner: &str, repo: &str, cache: &Cache) -> Self {
        Self {
            base_url: format!("https://api.github.com/repos/{owner}/{repo}"),
            base_key: format!("github-repo/{owner}/{repo}"),
            cache: cache.clone(),
        }
    }

    fn get<T: DeserializeOwned>(&self, url: String) -> Result<T, Error> {
        UrlFetch::new(&url)?.get()?.json()
    }

    pub fn get_info(&self) -> Result<GitHubRepoInfo, Error> {
        self.cache
            .get_or_insert(&format!("{}/info", self.base_key), 1.hour(), || {
                self.get(self.base_url.clone())
            })
    }

    pub fn get_branch_info(&self, branch: &str) -> Result<GitHubBranchInfo, Error> {
        self.cache
            .get_or_insert(&format!("{}/branch_info", self.base_key), 1.hour(), || {
                self.get(format!("{}/branches/{branch}", self.base_url))
            })
    }
}
