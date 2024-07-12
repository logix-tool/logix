use serde::{de::DeserializeOwned, Deserialize};

use crate::error::Error;

#[derive(Deserialize)]
pub struct GitHubOwner {
    //pub login: String,
    //pub url: String,
}

#[derive(Deserialize)]
pub struct GitHubRepoInfo {
    //pub name: String,
    //pub full_name: String,
    //pub private: bool,
    //pub owner: GitHubOwner,
    //pub fork: bool,
    pub default_branch: String,
}

#[derive(Deserialize)]
pub struct GitHubBranchInfo {
    //pub name: String,
    pub commit: GitHubCommit,
}

#[derive(Deserialize)]
pub struct GitHubCommit {
    pub sha: String,
    pub commit: GitHubCommitInfo,
}

#[derive(Deserialize)]
pub struct GitHubCommitInfo {
    pub author: GitHubCommitAuthor,
    //pub committer: GitHubCommitAuthor,
}

#[derive(Deserialize)]
pub struct GitHubCommitAuthor {
    //pub name: String,
    //pub email: String,
    #[serde(deserialize_with = "time::serde::rfc3339::deserialize")]
    pub date: time::OffsetDateTime,
}

pub struct GitHubRepo {
    base_url: String,
}

impl GitHubRepo {
    pub fn new(owner: &str, repo: &str) -> Self {
        Self {
            base_url: format!("https://api.github.com/repos/{owner}/{repo}"),
        }
    }

    fn get<T: DeserializeOwned>(&self, url: String) -> Result<T, Error> {
        match ureq::get(&url).call() {
            Ok(res) => {
                if res.status() == 200 {
                    res.into_json()
                        .map_err(|e| Error::HttpRequestJson(url, e.to_string()))
                } else {
                    Err(Error::HttpRequest(
                        url,
                        format!(
                            "Server returned status {} - {}",
                            res.status(),
                            res.status_text()
                        ),
                    ))
                }
            }
            Err(e) => Err(Error::HttpRequest(url, e.to_string())),
        }
    }

    pub fn get_info(&self) -> Result<GitHubRepoInfo, Error> {
        self.get(self.base_url.clone())
    }

    pub fn get_branch_info(&self, branch: &str) -> Result<GitHubBranchInfo, Error> {
        self.get(format!("{}/branches/{branch}", self.base_url))
    }
}
