use iced::widget::{button, column, container, row, text};
use iced::{Background, Border, Color, Element, Length, Task};
use std::fs;
use std::path::PathBuf;

fn geode_path_file() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(PathBuf::from))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("geode_path.txt")
}

pub fn load_geode_path() -> Option<String> {
    fs::read_to_string(geode_path_file())
        .ok()
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty())
}

pub fn save_geode_path(path: &str) -> std::io::Result<()> {
    fs::write(geode_path_file(), path.trim())
}

#[derive(Default)]
pub struct Setup {
    pub geode_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Browse,
    UseSystemDefault,
    PathPicked(Option<String>),
    Continue,
}

async fn pick_geode_path() -> Option<String> {
    rfd::AsyncFileDialog::new()
    .set_title("Select geode CLI executable")
    .pick_file()
    .await
    .map(|handle| handle.path().to_string_lossy().to_string())
}

impl Setup {

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Browse => Task::perform(pick_geode_path(), Message::PathPicked),
            Message::UseSystemDefault => {
                self.geode_path = Some("/usr/bin/geode".to_string());
                let _ = save_geode_path(&self.geode_path.clone().unwrap_or_default());
                Task::none()
            }
            Message::PathPicked(path) => {
                if let Some(path) = path {
                    self.geode_path = Some(path.clone());
                    let _ = save_geode_path(&path);
                }
                Task::none()
            }
            Message::Continue => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let path_label: Element<Message> = match &self.geode_path {
            Some(path) => text(path.clone()).size(13).into(),
            None => text("No path selected").size(13).into(),
        };

        let mut card = column![
            text("Setup").size(24),
            text("Locate your geode CLI executable to continue.").size(14),
            row![
                button(text("Browse").size(14))
                .on_press(Message::Browse)
                .padding([8, 16])
                .style(secondary_button_style),
                button(text("Use System Default (/usr/bin/geode)").size(14))
                .on_press(Message::UseSystemDefault)
                .padding([8, 16])
                .style(secondary_button_style),
            ]
            .spacing(10),
            container(path_label)
            .width(Length::Fill)
            .padding(10)
            .style(path_box_style),
        ]
        .spacing(18);

        if self.geode_path.is_some() {
            card = card.push(
                button(text("Continue").size(15))
                .on_press(Message::Continue)
                .width(Length::Fill)
                .padding(12)
                .style(primary_button_style),
            );
        }

        let card_box = container(card)
        .width(Length::Fixed(450.0))
        .padding(28)
        .style(card_style);

        container(card_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

fn card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x1F, 0x17, 0x2B))),
        border: Border {
            color: Color::from_rgb8(0x3A, 0x30, 0x48),
            width: 1.0,
            radius: 12.0.into(),
        },
        ..container::Style::default()
    }
}

fn path_box_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x0F, 0x0B, 0x16))),
        border: Border {
            color: Color::from_rgb8(0x3A, 0x30, 0x48),
            width: 1.0,
            radius: 6.0.into(),
        },
        text_color: Some(Color::from_rgb8(0xB0, 0xA8, 0xC0)),
        ..container::Style::default()
    }
}

fn primary_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb8(0x8A, 0x7C, 0xC0),
        button::Status::Pressed => Color::from_rgb8(0x5C, 0x50, 0x8C),
        _ => Color::from_rgb8(0x74, 0x66, 0xB0),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border {
            color: Color::from_rgb8(0x1A, 0x14, 0x24),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..button::Style::default()
    }
}

fn secondary_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb8(0x2E, 0x24, 0x3C),
        button::Status::Pressed => Color::from_rgb8(0x1A, 0x14, 0x24),
        _ => Color::from_rgb8(0x22, 0x1A, 0x2E),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::from_rgb8(0xE0, 0xDA, 0xEC),
        border: Border {
            color: Color::from_rgb8(0x3A, 0x30, 0x48),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..button::Style::default()
    }
}
