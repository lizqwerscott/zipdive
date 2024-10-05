use std::path::{Path, PathBuf};
use std::process::Command;

use walkdir::WalkDir;

use crate::error::Error;

fn is_compressed_file(file: &Path) -> bool {
    // 分为两种方法，一种使用后缀名，一种使用文件头

    let ext = file.extension().unwrap_or_default();
    let ext_matchp = matches!(
        ext.to_str(),
        Some("zip" | "rar" | "7z" | "tar" | "gz" | "bz2")
    );

    println!("file: {:?} {:?} {:?}", file, ext, ext_matchp);

    ext_matchp
}

pub fn collect_compressed_files_in_dir(search_dir: &Path) -> Result<Vec<PathBuf>, Error> {
    if !search_dir.exists() {
        return Err(Error::FileNotExists(search_dir.to_path_buf()));
    }

    let compressed_files = WalkDir::new(search_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.path().is_file() && is_compressed_file(entry.path()))
        .map(|entry| entry.path().to_path_buf())
        .collect();

    Ok(compressed_files)
}

pub async fn unzip_file(
    file_path: &Path,
    output_dir: &Path,
    password: Option<String>,
) -> Result<(), Error> {
    let output = match std::env::consts::OS {
        "windows" => {
            // bandzip
            let password_command = if let Some(password) = password {
                format!("-p:\"{}\"", password)
            } else {
                String::from("")
            };

            // TODO: 有密码的压缩文件如果不输入密码的话，不会报错，直接退出
            let command = format!(
                "Bandizip.exe x -y {} -o:\"{}\" \"{}\"",
                password_command,
                file_path.display(),
                output_dir.display()
            );

            Command::new("powershell.exe")
                .arg("-c")
                .arg(command)
                .output()?
        }
        "linux" | "macos" => {
            let mut command = format!(
                "7z x \"{}\" -o\"{}\"",
                file_path.display(),
                output_dir.display()
            );

            if let Some(password) = password {
                command = format!("{} -p \"{}\"", command, password);
            }

            // TODO: 有密码的压缩文件如果不输入密码的话，会卡住，需要处理并且报错
            Command::new("bash").arg("-c").arg(command).output()?
        }
        _ => {
            return Err(Error::SystemNotSupport);
        }
    };

    // linux
    if output.status.success() {
        Ok(())
    } else {
        Err(Error::ZipError((
            String::from_utf8_lossy(output.stderr.as_slice()).to_string(),
            file_path.to_path_buf(),
        )))
    }
}

pub fn change_path_root(old_root: &Path, path: &Path, new_root: &Path) -> PathBuf {
    let mut components = path.components();
    let mut old_root_components = old_root.components();
    while old_root_components.as_path() != Path::new("")
        && components
            .as_path()
            .starts_with(old_root_components.as_path())
    {
        components.next();
        old_root_components.next();
    }

    let output_path = new_root.join(components.as_path());

    output_path
}

#[cfg(test)]
mod zip_test {
    use super::*;

    use assert_fs::prelude::*;

    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::path::{Path, PathBuf};
    use zip::{result::ZipResult, write::FileOptions, CompressionMethod, ZipWriter};

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_collect_compressed() -> Result<(), Error> {
        let temp_project = assert_fs::TempDir::new().unwrap();
        let _archive_file = temp_project
            .child("test")
            .child("archive.tar.gz")
            .write_binary(&[]) // 创建一个空的 tar.gz 文件
            .unwrap();

        let found_files = collect_compressed_files_in_dir(temp_project.path())?;
        assert!(!found_files.is_empty());

        found_files.into_iter().for_each(|file| {
            let ext = file.extension().unwrap_or_default();
            assert_eq!(ext, "gz");
        });

        temp_project.close().unwrap();

        Ok(())
    }

    async fn unzip_dir(
        source_dir: &Path,
        target_dir: &Path,
        default_password: Option<String>,
    ) -> Result<(), Error> {
        let compressed_files = collect_compressed_files_in_dir(source_dir)?;

        for compressed_file in compressed_files {
            // generate new dir
            let file_base_name = compressed_file.file_stem().unwrap();

            let new_root_file = change_path_root(source_dir, &compressed_file, target_dir);
            let mut new_root_file_comp = new_root_file.components();
            new_root_file_comp.next_back();

            let output_dir = new_root_file_comp.as_path().join(file_base_name);
            println!("file: {:?} output_dir: {:?}", compressed_file, output_dir);
            std::fs::create_dir_all(&output_dir)?;

            // unzip to output_dir
            unzip_file(&compressed_file, &output_dir, default_password.clone()).await?;
        }

        Ok(())
    }

    fn create_zip_file(zip_file_path: &Path, files_to_compress: Vec<PathBuf>) -> ZipResult<()> {
        let zip_file = File::create(&zip_file_path)?;

        let mut zip = ZipWriter::new(zip_file);

        // Set compression options (e.g., compression method)
        let options = FileOptions::default().compression_method(CompressionMethod::DEFLATE);

        // Iterate through the files and add them to the ZIP archive.
        for file_path in &files_to_compress {
            let file = File::open(file_path)?;
            let file_name = file_path.file_name().unwrap().to_str().unwrap();

            // Adding the file to the ZIP archive.
            zip.start_file(file_name, options)?;

            let mut buffer = Vec::new();
            io::copy(&mut file.take(u64::MAX), &mut buffer)?;

            zip.write_all(&buffer)?;
        }

        zip.finish()?;

        Ok(())
    }

    #[test]
    fn test_unzip_dir() -> Result<(), Error> {
        if std::env::consts::OS == "windows" {
            return Ok(());
        }

        let temp_project = assert_fs::TempDir::new().unwrap();
        temp_project.child("source").create_dir_all().unwrap();
        temp_project.child("output").create_dir_all().unwrap();

        temp_project
            .child("source")
            .child("test_str.txt")
            .write_str("hello")
            .unwrap();

        let test_file = temp_project.path().join("source").join("test_str.txt");
        let zip_path = temp_project.path().join("source").join("file.zip");
        create_zip_file(&zip_path, vec![test_file]).unwrap();

        let source_dir = temp_project.path().join("source");
        let output_dir = temp_project.path().join("output");

        aw!(unzip_dir(&source_dir, &output_dir, None))?;

        let res_test_str_file = temp_project
            .path()
            .join("output")
            .join("file")
            .join("test_str.txt");
        println!("res_test_str_file: {}", res_test_str_file.display());
        assert!(res_test_str_file.exists());

        temp_project.close().unwrap();
        Ok(())
    }
}
