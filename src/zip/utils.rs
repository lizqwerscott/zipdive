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
            return Err(Error::SystemNotSupport);
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

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_collect_compressed() -> Result<(), Error> {
        let mut zip_dirs: Vec<&str> = Vec::new();
        zip_dirs.push("/home/lizqwer/TempProject/zipdive/source");

        for zip_dir in zip_dirs {
            let zip_dir = Path::new(zip_dir);
            let files = collect_compressed_files_in_dir(zip_dir)?;
            println!("zip_dir: {:?}", zip_dir);
            println!("{:?}", files);
        }

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

    #[test]
    fn test_unzip_dir() -> Result<(), Error> {
        let source_dir = Path::new("/home/lizqwer/TempProject/zipdive/source");
        let target_dir = Path::new("/home/lizqwer/TempProject/zipdive/output");

        aw!(unzip_dir(source_dir, target_dir, None))?;

        Ok(())
    }
}
