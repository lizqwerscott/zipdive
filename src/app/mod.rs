use std::fmt;
use std::path::PathBuf;

use iced::alignment::{Alignment, Horizontal};
use iced::widget::checkbox;
use iced::{
    widget::{button, center, column, container, row, text, text_input, Row},
    Element, Subscription, Task,
};
use rfd::FileDialog;

use crate::{error::Error, zip::Progress};

mod zipfiles;

use zipfiles::{ZipFiles, ZipsHandleState};

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

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    NeedInit,
    Running,
    Finish,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            State::NeedInit => write!(f, "需要初始化"),
            State::Running => write!(f, "正在解压"),
            State::Finish => write!(f, "解压完成"),
        }
    }
}

pub struct ZipDive {
    input_path: PathBuf,
    output_path: PathBuf,
    password: String,
    zip_files: Vec<ZipFiles>,
    now_run_zip_files: usize,
    auto_run: bool,
    state: State,
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
                state: State::NeedInit,
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
                self.input_path = path;
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
                self.output_path = path;
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
                match self.state {
                    State::Running => {
                        println!("已经处于运行状态");
                    }
                    _ => {
                        // check path exist
                        if !self.input_path.exists() || !self.output_path.exists() {
                            println!("路径不存在");
                            return Task::none();
                        }

                        self.now_run_zip_files = 1;
                        self.state = State::Running;

                        // 创建第一层的输出目录
                        let output_path = self.output_path.join("1");
                        self.zip_files.push(ZipFiles::new(
                            self.input_path.clone(),
                            output_path,
                            self.now_run_zip_files,
                        ));
                    }
                }

                Task::none()
            }
            Message::Next => {
                match self.state {
                    State::Finish => {
                        println!("递归解压完成");
                    }
                    State::Running => {
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
                    }
                    State::NeedInit => {
                        println!("需要开始解压");
                    }
                }

                Task::none()
            }
            Message::ZipFileHandleProgress((id, progress)) => {
                if let Some(zip_file) = self.zip_files.get_mut(id - 1) {
                    zip_file.progress(progress);

                    match zip_file.state {
                        ZipsHandleState::Finished => {
                            if self.auto_run {
                                self.next_zip_files();
                            }
                        }
                        ZipsHandleState::EmptyZips => {
                            self.state = State::Finish;
                        }
                        _ => {}
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
        let input_path_input_helper = text_input(
            "输入要处理的文件路径...",
            self.input_path.display().to_string().as_str(),
        );
        let input_path_button_helper = button("select");

        let (input_path_input, input_path_button) = if self.state == State::Running {
            (input_path_input_helper, input_path_button_helper)
        } else {
            (
                input_path_input_helper.on_input(Message::InputPathChange),
                input_path_button_helper.on_press(Message::InputPathFileDialog),
            )
        };

        let output_path_input_helper = text_input(
            "输入要导出的位置...",
            &self.output_path.display().to_string().as_str(),
        );
        let input_path_button_helper = button("select");

        let (output_path_input, output_path_button) = if self.state == State::Running {
            (output_path_input_helper, input_path_button_helper)
        } else {
            (
                output_path_input_helper.on_input(Message::OutputPathChange),
                input_path_button_helper.on_press(Message::OutputPathFileDialog),
            )
        };

        let password_input =
            text_input("输入默认压缩密码...", &self.password).on_input(Message::PasswordChange);

        let start_button = button("Start").on_press(Message::Start);
        let next_button = button("Next").on_press(Message::Next);
        let auto_run_checkbox =
            checkbox("AutoRun", self.auto_run).on_toggle(Message::AutoRunCheckboxToggled);

        let state_show = text(format!("状态: {}", self.state)).shaping(text::Shaping::Advanced);

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
                row![state_show, start_button, next_button, auto_run_checkbox]
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
            Row::with_children(self.zip_files.iter().map(ZipFiles::view))
                .spacing(10)
                .into()
        };

        container(column![controls, show_zip_files].spacing(10))
            .padding(10)
            .into()
    }
}
