use std::path::PathBuf;

use iced::futures::{SinkExt, Stream, StreamExt};
use iced::stream::try_channel;
use iced::Subscription;

use crate::error::Error;

mod utils;

use utils::{change_path_root, collect_compressed_files_in_dir, unzip_file};

#[derive(Debug, Clone)]
pub enum Progress {
    EmptyZips,
    Searching { zip_files: Vec<PathBuf> },
    Zipping { file: PathBuf },
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

        for compressed_file in compressed_files {
            // generate new dir
            let file_base_name = compressed_file.file_stem().unwrap();

            let new_root_file = change_path_root(&source_dir, &compressed_file, &target_dir);
            let mut new_root_file_comp = new_root_file.components();
            new_root_file_comp.next_back();

            let output_dir = new_root_file_comp.as_path().join(file_base_name);
            println!("file: {:?} output_dir: {:?}", compressed_file, output_dir);
            std::fs::create_dir_all(&output_dir)?;

            // unzip to output_dir
            unzip_file(&compressed_file, &output_dir, default_password.clone()).await?;

            let _ = output
                .send(Progress::Zipping {
                    file: compressed_file,
                })
                .await;
        }

        let _ = output.send(Progress::Finished).await;

        Ok(())
    })
}
