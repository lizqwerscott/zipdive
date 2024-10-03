use std::fmt;
use std::path::PathBuf;

use iced::alignment::{Alignment, Horizontal};
use iced::widget::checkbox;
use iced::{
    widget::{button, center, column, container, row, text, text_input, Column, Row},
    Element, Length, Subscription, Task,
};
use rfd::FileDialog;

use crate::{error::Error, zip::run_zip_dir, zip::Progress};

#[derive(Clone, Debug)]
pub enum Message {
    InputPathChange(String),
    OutputPathChange(String),
    InputPathFileDialog,
    OutputPathFileDialog,
    PasswordChange(String),
    Start,
    ZipFileHandleProgress((usize, Result<Progress, Error>)),
    Next,
    AutoRunCheckboxToggled(bool),
}

struct ZipFile {
    name: String,
    path: PathBuf,
}

impl ZipFile {
    fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }

    fn view(&self) -> Element<Message> {
        text(format!(
            "{}: {}",
            self.name,
            self.path.display().to_string()
        ))
        .width(Length::Fill)
        .shaping(text::Shaping::Advanced)
        .into()
    }
}

struct ZipFiles {
    input_path: PathBuf,
    output_path: PathBuf,
    zip_files: Vec<ZipFile>,
    depth: usize,
    state: ZipsHandleState,
}

#[derive(Clone, Debug)]
enum ZipsHandleState {
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
    fn new(input_path: PathBuf, output_path: PathBuf, depth: usize) -> Self {
        Self {
            input_path,
            output_path,
            zip_files: Vec::new(),
            depth,
            state: ZipsHandleState::Searching,
        }
    }

    fn progress(&mut self, new_progress: Result<Progress, Error>) {
        match self.state {
            ZipsHandleState::Searching | ZipsHandleState::Zipping => match new_progress {
                Ok(progress) => match progress {
                    Progress::Finished => self.state = ZipsHandleState::Finished,
                    Progress::Zipping { file } => {}
                    Progress::EmptyZips => {
                        self.state = ZipsHandleState::EmptyZips;
                    }
                    Progress::Searching { zip_files } => {
                        for zip_file in zip_files {
                            let file_name = zip_file.file_name().unwrap().to_str().unwrap();

                            self.zip_files
                                .push(ZipFile::new(String::from(file_name), zip_file));
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

    fn view(&self) -> Element<Message> {
        let title_str = format!(
            "第 {} 层: 压缩文件数量: {} 状态: {}",
            self.depth,
            self.zip_files.len(),
            self.state.to_string()
        );

        let deepth_title = text(title_str).shaping(text::Shaping::Advanced);

        let zip_files = Column::with_children(self.zip_files.iter().map(ZipFile::view));

        column![deepth_title, zip_files].into()
    }
}

pub struct ZipDive {
    input_path: PathBuf,
    output_path: PathBuf,
    password: String,
    zip_files: Vec<ZipFiles>,
    now_run_zip_files: usize,
    auto_run: bool,
    all_finish: bool,
}

impl ZipDive {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                input_path: PathBuf::from("/home/lizqwer/TempProject/zipdive/source"),
                output_path: PathBuf::from("/home/lizqwer/TempProject/zipdive/output"),
                password: String::from(""),
                zip_files: Vec::new(),
                now_run_zip_files: 0,
                auto_run: false,
                all_finish: false,
            },
            Task::none(),
        )
    }

    fn next_zip_files(&mut self) {
        self.now_run_zip_files += 1;

        let input_path = self
            .output_path
            .join(format!("{}", self.now_run_zip_files - 1));
        let output_path = self.output_path.join(format!("{}", self.now_run_zip_files));
        self.zip_files.push(ZipFiles::new(
            input_path,
            output_path,
            self.now_run_zip_files,
        ));
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputPathChange(s) => {
                let path = PathBuf::from(s.trim());
                if path.exists() {
                    self.input_path = path;
                }
                Task::none()
            }
            Message::InputPathFileDialog => {
                let file = FileDialog::new().pick_folder();
                if let Some(file) = file {
                    self.input_path = file;
                }
                Task::none()
            }
            Message::OutputPathChange(s) => {
                let path = PathBuf::from(s.trim());
                if path.exists() {
                    self.output_path = path;
                }
                Task::none()
            }
            Message::OutputPathFileDialog => {
                let file = FileDialog::new().pick_folder();
                if let Some(file) = file {
                    self.output_path = file;
                }
                Task::none()
            }
            Message::PasswordChange(password) => {
                self.password = password;
                Task::none()
            }
            Message::Start => {
                if self.now_run_zip_files > 0 {
                    println!("已经处于运行状态");
                } else {
                    self.now_run_zip_files = 1;

                    // 创建第一层的输出目录
                    let output_path = self.output_path.join("1");
                    self.zip_files.push(ZipFiles::new(
                        self.input_path.clone(),
                        output_path,
                        self.now_run_zip_files,
                    ));
                }

                Task::none()
            }
            Message::Next => {
                if !self.all_finish {
                    if !self.auto_run {
                        if let Some(last_zip_files) = self.zip_files.last() {
                            match last_zip_files.state {
                                ZipsHandleState::Finished => {
                                    self.next_zip_files();
                                }
                                ZipsHandleState::EmptyZips => {
                                    println!("已经搜索到最后一层，无法进行下一层解压");
                                }
                                _ => {
                                    println!("上一层解压未完成，无法进行下一层解压");
                                }
                            }
                        }
                    } else {
                        println!("处于自动运行模式，无需手动操作");
                    }
                } else {
                    println!("递归解压完成");
                }

                Task::none()
            }
            Message::ZipFileHandleProgress((id, progress)) => {
                if let Some(zip_file) = self.zip_files.get_mut(id - 1) {
                    zip_file.progress(progress);
                    if self.auto_run {
                        match zip_file.state {
                            ZipsHandleState::Finished => {
                                self.next_zip_files();
                            }
                            _ => {}
                        }
                    }
                }
                Task::none()
            }
            Message::AutoRunCheckboxToggled(auto_run) => {
                self.auto_run = auto_run;
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(self.zip_files.iter().map(ZipFiles::subscription))
    }

    pub fn view(&self) -> Element<Message> {
        let input_path_input = text_input(
            "输入要处理的文件路径...",
            self.input_path.display().to_string().as_str(),
        )
        .on_input(Message::InputPathChange)
        .padding(10);
        let input_path_button = button("select")
            .padding(10)
            .on_press(Message::InputPathFileDialog);

        let output_path_input = text_input(
            "输入要导出的位置...",
            &self.output_path.display().to_string().as_str(),
        )
        .on_input(Message::OutputPathChange)
        .padding(10);

        let output_path_button = button("select")
            .padding(10)
            .on_press(Message::OutputPathFileDialog);

        let password_input = text_input("输入默认压缩密码...", &self.password)
            .on_input(Message::PasswordChange)
            .padding(10);

        let start_button = button("Start").on_press(Message::Start);
        let next_button = button("Next").on_press(Message::Next);
        let auto_run_checkbox =
            checkbox("AutoRun", self.auto_run).on_toggle(Message::AutoRunCheckboxToggled);

        let controls = row![
            column![
                row![
                    text("压缩文件目录:").shaping(text::Shaping::Advanced),
                    input_path_input,
                    input_path_button
                ]
                .align_y(Alignment::Center)
                .spacing(10),
                row![
                    text("解压到的目录:").shaping(text::Shaping::Advanced),
                    output_path_input,
                    output_path_button
                ]
                .align_y(Alignment::Center)
                .spacing(10)
            ]
            .spacing(10),
            column![
                row![
                    text("解压密码:").shaping(text::Shaping::Advanced),
                    password_input.padding(10)
                ]
                .align_y(Alignment::Center)
                .spacing(10),
                row![start_button, next_button, auto_run_checkbox]
                    .align_y(Alignment::Center)
                    .spacing(10)
            ]
            .align_x(Horizontal::Center)
            .spacing(10),
        ]
        .spacing(10);

        let show_zip_files: Element<Message> = if self.zip_files.is_empty() {
            center(text("没有压缩文件").shaping(text::Shaping::Advanced)).into()
        } else {
            Row::with_children(self.zip_files.iter().map(ZipFiles::view)).into()
        };

        container(column![controls, show_zip_files])
            .padding(10)
            .into()
    }
}
