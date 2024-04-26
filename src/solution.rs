use std::time::Duration;

use crate::statement::{self, *};
use async_trait::async_trait;
use tokio::{sync::mpsc, time::sleep};

#[async_trait]
pub trait Solution {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary>;
}

pub struct Solution0;

#[async_trait]
impl Solution for Solution0 {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary> {
        let (downloaded_tx, mut rx) = mpsc::channel(repositories.len());
        let mut tasks = vec![];
        let l = repositories.len();

        for repo in repositories {
            let downloaded_tx = downloaded_tx.clone();
            let task = tokio::spawn(async move {
                let mut i = 0;
                let mut delay_secs = INITIAL_RETRY_DELAY_SECS;
                loop {
                    let res = statement::download(repo.clone()).await;
                    match res {
                        Ok(b) => {
                            println!("downloaded {repo:?}");
                            let _ = downloaded_tx.send(Ok((b, repo))).await;
                            break;
                        }
                        Err(e) => {
                            i += 1;
                            if i >= RETRY_COUNT {
                                let _ = downloaded_tx.send(Err(e)).await;
                                break;
                            } else {
                                println!("retry {repo:?}");
                                sleep(Duration::from_secs(delay_secs)).await;
                                delay_secs *= 2;
                            }
                        }
                    }
                }
            });
            tasks.push(task);
        }

        let mut res = None;
        let mut a = 0;

        while let Some(result) = rx.recv().await {
            match result {
                Ok((a, repo)) => {
                    println!("install {repo:?}");
                    res = Some(a);
                    tasks.iter().for_each(|t| t.abort());
                    break;
                }
                Err(e) => {
                    a += 1;
                    println!("Download error: {e:?}");
                    if a == l {
                        break;
                    }
                }
            }
        }
        res
    }
}

const RETRY_COUNT: u32 = 3;
const INITIAL_RETRY_DELAY_SECS: u64 = 2;
