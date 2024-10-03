use std::path::PathBuf;

use iced::futures::{SinkExt, Stream, StreamExt};
use iced::stream::try_channel;
use iced::Subscription;
use tokio::task::JoinSet;

use crate::error::Error;

mod utils;

use utils::{change_path_root, collect_compressed_files_in_dir, unzip_file};

#[derive(Debug, Clone)]
pub enum Progress {
    EmptyZips,
    Searching {
        zip_files: Vec<PathBuf>,
    },
    Zipping {
        file_id: usize,
        state: Result<(), Error>,
    },
    Finished,
}

pub fn run_zip_dir(
    id: usize,
    source_dir: PathBuf,
    target_dir: PathBuf,
    default_password: Option<String>,
) -> iced::Subscription<(usize, Result<Progress, Error>)> {
    Subscription::run_with_id(
        id,
        unzip_dir_s(source_dir, target_dir, default_password).map(move |progress| (id, progress)),
    )
}

fn unzip_dir_s(
    source_dir: PathBuf,
    target_dir: PathBuf,
    default_password: Option<String>,
) -> impl Stream<Item = Result<Progress, Error>> {
    try_channel(1, move |mut output| async move {
        let compressed_files = collect_compressed_files_in_dir(&source_dir)?;
        if compressed_files.is_empty() {
            let _ = output.send(Progress::EmptyZips).await;

            return Ok(());
        }
        let _ = output
            .send(Progress::Searching {
                zip_files: compressed_files.clone(),
            })
            .await;

        let mut set = JoinSet::new();

        for (index, compressed_file) in compressed_files.into_iter().enumerate() {
            // generate new dir
            let file_base_name = compressed_file.file_stem().unwrap();

            let new_root_file = change_path_root(&source_dir, &compressed_file, &target_dir);
            let mut new_root_file_comp = new_root_file.components();
            new_root_file_comp.next_back();

            let output_dir = new_root_file_comp.as_path().join(file_base_name);
            println!("file: {:?} output_dir: {:?}", compressed_file, output_dir);
            std::fs::create_dir_all(&output_dir)?;

            let password = default_password.clone();
            set.spawn(async move {
                (
                    index,
                    unzip_file(&compressed_file, &output_dir, password).await,
                )
            });
        }

        while let Some(res) = set.join_next().await {
            let res = res.unwrap();
            let _ = output
                .send(Progress::Zipping {
                    file_id: res.0,
                    state: res.1,
                })
                .await;
        }

        let _ = output.send(Progress::Finished).await;

        Ok(())
    })
}
