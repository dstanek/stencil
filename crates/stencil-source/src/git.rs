// Copyright 2024-2025 David Stanek <dstanek@dstanek.com>

use std::collections::VecDeque;
use std::env;

use serde::Deserialize;
use ureq::Error;

use super::model::{Directory, File, Renderable};
use stencil_error::StencilError;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum GitHubItemType {
    File,
    Dir,
}

#[derive(Deserialize)]
struct GitHubItem {
    path: String,
    #[serde(rename = "type")]
    item_type: GitHubItemType,
    download_url: Option<String>,
}

pub struct GithubRepoIterator {
    owner: String,
    repo: String,
    path: String,
    token: Option<String>,
    queue: VecDeque<GitHubItem>,
}

fn get_directory_contents(
    owner: &str,
    repo: &str,
    path: &str,
    token: Option<&str>,
) -> Result<Vec<GitHubItem>, Error> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        owner, repo, path
    );
    let request = ureq::get(&url).header("User-Agent", "stencil");
    let mut response = if let Some(token) = token {
        request
            .header("Authorization", &format!("Bearer {}", token))
            .call()?
    } else {
        request.call()?
    };
    let response_text = response.body_mut().read_to_string()?;

    let items: Vec<GitHubItem> = serde_json::from_str(&response_text)?;
    Ok(items)
}

fn get_file_content(url: &str, token: Option<&str>) -> Result<Option<String>, Error> {
    let request = ureq::get(url).header("User-Agent", "stencil");
    let mut response = if let Some(token) = token {
        request
            .header("Authorization", &format!("Bearer {}", token))
            .call()?
    } else {
        request.call()?
    };

    if response.status() == 200 {
        let content = response.body_mut().read_to_string()?;
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

impl GithubRepoIterator {
    pub fn new(
        owner: String,
        repo: String,
        path: String,
        //) -> Result<GithubRepoIterator, StencilError> {
    ) -> Result<Self, StencilError> {
        let token = env::var("GITHUB_TOKEN").ok();
        let items = get_directory_contents(&owner, &repo, &path, token.as_deref()).unwrap();
        Ok(GithubRepoIterator {
            owner,
            repo,
            path: path.clone(),
            token,
            queue: VecDeque::from(items),
        })
    }
}

impl Iterator for GithubRepoIterator {
    type Item = Result<Renderable, StencilError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.queue.pop_front() {
            let relative_path = if item.path.starts_with(&self.path) {
                let start = self.path.len();
                item.path[start..].trim_start_matches('/').to_string()
            } else {
                item.path.to_string()
            };
            match item.item_type {
                GitHubItemType::File => {
                    if let Some(url) = item.download_url {
                        match get_file_content(&url, self.token.as_deref()) {
                            Ok(Some(content)) => {
                                return Some(Ok(Renderable::File(File::new(
                                    relative_path,
                                    content,
                                ))));
                            }
                            Ok(None) => continue,
                            Err(e) => {
                                println!("Error fetching file content: {}", e);
                                return Some(Err(StencilError::Other(e.to_string())));
                            }
                        }
                    }
                }
                GitHubItemType::Dir => {
                    let items = match get_directory_contents(
                        &self.owner,
                        &self.repo,
                        &item.path,
                        self.token.as_deref(),
                    ) {
                        Ok(items) => items,
                        Err(e) => {
                            println!("Error fetching directory contents: {}", e);
                            return Some(Err(StencilError::Other(e.to_string())));
                        }
                    };
                    for item in items.into_iter().rev() {
                        self.queue.push_front(item);
                    }

                    return Some(Ok(Renderable::Directory(Directory::new(relative_path))));
                }
            }
        }
        None
    }
}
