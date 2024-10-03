use iced::{
    font::{Family, Weight},
    Font, Settings,
};

use zipdive::app::ZipDive;

fn main() -> iced::Result {
    let settings = Settings {
        default_font: Font {
            family: Family::Name("LXGW WenKai".into()),
            weight: Weight::Normal,
            ..Default::default()
        },
        ..Settings::default()
    };

    iced::application("A cool zip derive", ZipDive::update, ZipDive::view)
        .settings(settings)
        .subscription(ZipDive::subscription)
        .run_with(ZipDive::new)
}
