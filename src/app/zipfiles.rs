use std::fmt;
use std::path::{Path, PathBuf};

use iced::alignment::Alignment;
use iced::widget::checkbox;
use iced::{
    widget::{column, progress_bar, row, text, Column},
    Element, Length, Subscription,
};

use crate::{error::Error, zip::run_zip_dir, zip::Progress};

use super::Message;

struct ZipFile {
    show_path: PathBuf,
    state: ZipFileHandleState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ZipFileHandleState {
    Running,
    Finished,
    Error,
}

impl ZipFile {
    fn new(path: PathBuf, parent: PathBuf) -> Self {
        let mut components = path.components();
        let mut parent_components = parent.components();

        while parent_components.as_path() != Path::new("")
            && components
                .as_path()
                .starts_with(parent_components.as_path())
        {
            components.next();
            parent_components.next();
        }

        Self {
            show_path: components.as_path().to_path_buf(),
            state: ZipFileHandleState::Running,
        }
    }

    fn view(&self) -> Element<Message> {
        let start_icon: Element<Message> = match self.state {
            ZipFileHandleState::Running | ZipFileHandleState::Finished => {
                checkbox("", self.state == ZipFileHandleState::Finished).into()
            }
            ZipFileHandleState::Error => text("❌").shaping(text::Shaping::Advanced).into(),
        };

        row![
            start_icon,
            text(format!("{}", self.show_path.display(),))
                .width(Length::Fill)
                .shaping(text::Shaping::Advanced),
        ]
        .into()
    }
}

pub struct ZipFiles {
    input_path: PathBuf,
    output_path: PathBuf,
    zip_files: Vec<ZipFile>,
    depth: usize,
    pub state: ZipsHandleState,
    finish_count: usize,
}

#[derive(Clone, Debug)]
pub enum ZipsHandleState {
    Searching,
    Zipping,
    Finished,
    Error,
    EmptyZips,
}

impl fmt::Display for ZipsHandleState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZipsHandleState::Searching => write!(f, "搜索中"),
            ZipsHandleState::Zipping => write!(f, "解压中"),
            ZipsHandleState::Finished => write!(f, "完成"),
            ZipsHandleState::Error => write!(f, "错误"),
            ZipsHandleState::EmptyZips => write!(f, "没有压缩文件"),
        }
    }
}

impl ZipFiles {
    pub fn new(input_path: PathBuf, output_path: PathBuf, depth: usize) -> Self {
        Self {
            input_path,
            output_path,
            zip_files: Vec::new(),
            depth,
            state: ZipsHandleState::Searching,
            finish_count: 0,
        }
    }

    pub fn progress(&mut self, new_progress: Result<Progress, Error>) {
        match self.state {
            ZipsHandleState::Searching | ZipsHandleState::Zipping => match new_progress {
                Ok(progress) => match progress {
                    Progress::Finished => self.state = ZipsHandleState::Finished,
                    Progress::Zipping { file_id, state } => {
                        match state {
                            Ok(()) => {
                                self.zip_files[file_id].state = ZipFileHandleState::Finished;
                            }
                            Err(e) => {
                                self.zip_files[file_id].state = ZipFileHandleState::Error;
                                println!("Error: {}", e);
                            }
                        }

                        self.finish_count += 1;
                    }
                    Progress::EmptyZips => {
                        self.state = ZipsHandleState::EmptyZips;
                    }
                    Progress::Searching { zip_files } => {
                        for zip_file in zip_files {
                            self.zip_files
                                .push(ZipFile::new(zip_file, self.input_path.clone()));
                        }
                        self.state = ZipsHandleState::Zipping;
                    }
                },
                Err(_error) => self.state = ZipsHandleState::Error,
            },
            _ => {}
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self.state {
            ZipsHandleState::Searching | ZipsHandleState::Zipping => run_zip_dir(
                self.depth,
                self.input_path.clone(),
                self.output_path.clone(),
                None,
            )
            .map(Message::ZipFileHandleProgress),
            _ => Subscription::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        // TODO: 每一层输出目录提供打开和复制
        let title_str = format!("第 {} 层: {}", self.depth, self.state.to_string(),);

        let path_str = format!("{}", self.input_path.display());

        let deepth_title = row![
            text(title_str).shaping(text::Shaping::Advanced),
            progress_bar(0.0..=self.zip_files.len() as f32, self.finish_count as f32),
            text(format!("{}/{}", self.finish_count, self.zip_files.len()))
                .shaping(text::Shaping::Advanced)
        ]
        .align_y(Alignment::Center)
        .spacing(3);

        let deepth_path = text(path_str).shaping(text::Shaping::Advanced);

        let zip_files = Column::with_children(self.zip_files.iter().map(ZipFile::view)).spacing(5);

        column![deepth_title, deepth_path, zip_files].into()
    }
}
