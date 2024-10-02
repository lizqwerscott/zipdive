use druid::widget::{prelude::*, CrossAxisAlignment, MainAxisAlignment};
use druid::widget::{Button, Flex, Label, TextBox};
use druid::{
    commands, AppDelegate, AppLauncher, Color, Command, DelegateCtx, FileDialogOptions, FileInfo,
    Handled, LensExt, Selector, Target, Widget, WidgetExt, WindowDesc,
};
use druid::{Data, Lens};

use zipdive::collect_compressed_files_in_dir;

const TEXT_SIZE: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 300.0;
const SELECT_ZIP_DIR: Selector<FileInfo> = Selector::new("SELECT_ZIP_DIR");
const SELECT_OUTPUT_DIR: Selector<FileInfo> = Selector::new("SELECT_OUTPUT_DIR");

#[derive(Clone, Eq, PartialEq, Data)]
enum RunState {
    NotRun,
    Running,
    Pause,
    Finish,
}

#[derive(Clone, Data, Lens)]
struct GuiState {
    zip_dir: String,
    output_dir: String,
    zip_password: String,
    run_state: RunState,
}

impl GuiState {
    fn default() -> Self {
        GuiState {
            zip_dir: String::default(),
            output_dir: String::default(),
            zip_password: String::default(),
            run_state: RunState::NotRun,
        }
    }
}

struct Delegate;

impl AppDelegate<GuiState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut GuiState,
        _env: &Env,
    ) -> Handled {
        // if let Some(file_info) = cmd.get(commands::SAVE_FILE_AS) {
        // }

        if let Some(file_info) = cmd.get(SELECT_ZIP_DIR) {
            if let Some(file_path) = file_info.path().as_os_str().to_str() {
                data.zip_dir = file_path.to_string();
            }

            return Handled::Yes;
        }

        if let Some(file_info) = cmd.get(SELECT_OUTPUT_DIR) {
            if let Some(file_path) = file_info.path().as_os_str().to_str() {
                data.output_dir = file_path.to_string();
            }

            return Handled::Yes;
        }
        Handled::No
    }
}

fn build_root_widget() -> impl Widget<GuiState> {
    let zip_dir_textbox = TextBox::new()
        .with_placeholder("输入压缩文件目录")
        .with_text_size(TEXT_SIZE)
        .fix_width(TEXT_BOX_WIDTH)
        .lens(GuiState::zip_dir);

    let open_dialog_options = FileDialogOptions::new()
        // .name_label("")
        .title("选择压缩文件目录")
        .select_directories()
        .button_text("Import")
        .accept_command(SELECT_ZIP_DIR);

    let open = Button::new("Selct").on_click(move |ctx, _, _| {
        ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_dialog_options.clone()))
    });

    let output_text_box = TextBox::new()
        .with_placeholder("输入解压输出目录")
        .with_text_size(TEXT_SIZE)
        .fix_width(TEXT_BOX_WIDTH)
        .lens(GuiState::output_dir);

    let open_dialog_options = FileDialogOptions::new()
        // .name_label("")
        .title("选择解压输出目录")
        .select_directories()
        .button_text("Select")
        .accept_command(SELECT_OUTPUT_DIR);

    let output_open = Button::new("Selct").on_click(move |ctx, _, _| {
        ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(open_dialog_options.clone()))
    });

    let first_left_column = Flex::row()
        // .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(
            Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::End)
                .with_child(Label::new("压缩文件目录").with_text_size(TEXT_SIZE))
                .with_child(Label::new("输出目录").with_text_size(TEXT_SIZE)),
        )
        .with_child(
            Flex::column()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(
                    Flex::row()
                        .with_child(Label::new(":").with_text_size(TEXT_SIZE))
                        .with_child(zip_dir_textbox)
                        .with_child(open),
                )
                .with_child(
                    Flex::row()
                        .with_child(Label::new(":").with_text_size(TEXT_SIZE))
                        .with_child(output_text_box)
                        .with_child(output_open),
                ),
        );

    let password_text_box = TextBox::new()
        .with_placeholder("输入解压密码")
        .with_text_size(TEXT_SIZE)
        .fix_width(TEXT_BOX_WIDTH)
        .lens(GuiState::zip_password);

    let start_unzip = Button::new("开始").on_click(move |ctx, _, _| {});

    let pause_unzip = Button::new("暂停").on_click(move |ctx, _, _| {});

    let stop_unzip = Button::new("停止").on_click(move |ctx, _, _| {});

    let button_column = Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .with_default_spacer()
        .with_child(start_unzip)
        .with_child(pause_unzip)
        .with_child(stop_unzip);

    let second_column = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(
            Flex::row()
                // .cross_axis_alignment(CrossAxisAlignment::Start)
                .with_default_spacer()
                .with_child(Label::new("解压密码:").with_text_size(TEXT_SIZE))
                .with_child(password_text_box),
        )
        // .with_flex_spacer(1.0)
        .with_child(button_column);

    let first_row = Flex::row()
        // .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(first_left_column)
        .with_child(second_column);

    let second_row = Flex::row().with_child(Label::new("状态:").with_text_size(TEXT_SIZE));
    // column
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(first_row)
        .with_child(second_row)
}

fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title("ZipDrive")
        .window_size((900.0, 800.0));

    let initial_state = GuiState::default();

    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .log_to_console()
        .launch(initial_state)
        .expect("launch failed");
}
