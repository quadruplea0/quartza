mod setup;
mod profiles;
mod new;
mod build;

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{theme, Background, Border, Color, Element, Font, Length, Task};
use iced_aw::style::status::Status;
use iced_aw::style::tab_bar::Style;
use iced_aw::tab_bar::TabLabel;
use iced_aw::TabBar;

fn main() -> iced::Result {
    iced::application(State::default, State::update, State::view)
    .title("Quartz")
    .style(bg_style)
    .window(iced::window::Settings {
        size: iced::Size::new(900.0, 600.0),
            resizable: false,
            ..Default::default()
    })
    .run()
}

#[derive(Debug, Clone)]
enum Message {
    Setup(setup::Message),
    App(AppMessage),
}

enum State {
    Setup(setup::Setup),
    Main(quartz),
}

impl Default for State {
    fn default() -> Self {
        if let Some(path) = setup::load_geode_path() {
            State::Main(quartz::new(path))
        } else {
            State::Setup(setup::Setup::default())
        }
    }
}



impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Setup(setup::Message::Continue) => {
                if let State::Setup(setup) = self {
                    if let Some(path) = setup.geode_path.clone() {
                        *self = State::Main(quartz::new(path));
                    }
                }
                Task::none()
            }
            Message::Setup(msg) => {
                if let State::Setup(setup) = self {
                    setup.update(msg).map(Message::Setup)
                } else {
                    Task::none()
                }
            }
            Message::App(msg) => {
                if let State::Main(app) = self {
                    app.update(msg).map(Message::App)
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            State::Setup(setup) => setup.view().map(Message::Setup),
            State::Main(app) => app.view().map(Message::App),
        }
    }
}

fn bg_style(_state: &State, _theme: &iced::Theme) -> theme::Style {
    theme::Style {
        background_color: Color::from_rgb8(0x19, 0x11, 0x25),
        text_color: Color::WHITE,
    }
}

#[derive(Debug, Clone)]
enum AppMessage {
    TabSelected(usize),
    RunGeodeVersion,
    CommandOutput(String),
    ToggleTty,
    Profiles(profiles::Message),
    New(new::Message),
    Build(build::Message),
}

struct quartz {
    geode_path: String,
    selected_tab: usize,
    tty_output: String,
    tty_expanded: bool,
    profiles: profiles::Tab,
    new: new::Tab,
    build: build::Tab,
}

impl quartz {
    fn new(geode_path: String) -> Self {
        Self {
            geode_path,
            selected_tab: 0,
            tty_output: String::new(),
            tty_expanded: false,
            profiles: profiles::Tab::default(),
            new: new::Tab::default(),
            build: build::Tab::default(),
        }
    }
}

async fn run_command(program: String, args: Vec<String>) -> String {
    match std::process::Command::new(program).args(&args).output() {
        Ok(output) => {
            let mut result = String::from_utf8_lossy(&output.stdout).to_string();
            result.push_str(&String::from_utf8_lossy(&output.stderr));
            result
        }
        Err(e) => format!("command failed: {e}"),
    }
}

fn tabsstyle(theme: &iced::Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let (bg, border) = match status {
        Status::Active => (Color::from_rgb8(0x6E, 0x64, 0x7C), Color::from_rgb8(0x1A, 0x14, 0x24)),
        _ => (Color::from_rgb8(0x5A, 0x51, 0x68), Color::from_rgb8(0x1A, 0x14, 0x24)),
    };
    Style {
        background: Some(Background::Color(Color::from_rgb8(0x19, 0x11, 0x25))),
        border_color: Some(border),
        border_width: 3.0,
        tab_label_background: Background::Color(bg),
        tab_label_border_color: border,
        tab_label_border_width: 3.0,
        tab_border_radius: 12.0.into(),
        icon_background: None,
        icon_border_radius: 0.0.into(),
        icon_color: palette.background.base.text,
        text_color: palette.background.base.text,
    }
}

fn panelstyle(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x22, 0x1A, 0x2E))),
        border: Border {
            color: Color::from_rgb8(0x3A, 0x30, 0x48),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

fn textboxstyle(_theme: &iced::Theme, status: iced::widget::text_input::Status) -> iced::widget::text_input::Style {
    let bordercolor = match status {
        iced::widget::text_input::Status::Focused { .. } => Color::from_rgb8(0x74, 0x66, 0xB0),
        _ => Color::from_rgb8(0x3A, 0x30, 0x48),
    };
    iced::widget::text_input::Style {
        background: Background::Color(Color::from_rgb8(0x22, 0x1A, 0x2E)),
        border: Border { color: bordercolor, width: 1.0, radius: 6.0.into() },
        icon: Color::from_rgb8(0x6E, 0x64, 0x7C),
        placeholder: Color::from_rgb8(0x6E, 0x64, 0x7C),
        value: Color::WHITE,
        selection: Color::from_rgb8(0x5C, 0x50, 0x8C),
    }
}

fn ttystyle(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x0F, 0x0A, 0x18))),
        border: Border {
            color: Color::from_rgb8(0x1A, 0x14, 0x24),
            width: 2.0,
            radius: 6.0.into(),
        },
        text_color: Some(Color::WHITE),
        ..container::Style::default()
    }
}

fn buttonstyle1(_theme: &iced::Theme, status: button::Status) -> button::Style {
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

fn buttonstyle2(_theme: &iced::Theme, status: button::Status) -> button::Style {
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

fn togglestyle(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb8(0x2E, 0x24, 0x3C),
        _ => Color::from_rgb8(0x22, 0x1A, 0x2E),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::from_rgb8(0xE0, 0xDA, 0xEC),
        border: Border {
            color: Color::from_rgb8(0x3A, 0x30, 0x48),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
    }
}

impl quartz {
    fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            AppMessage::TabSelected(index) => {
                self.selected_tab = index;
                Task::none()
            }
            AppMessage::RunGeodeVersion => Task::perform(
                run_command(self.geode_path.clone(), vec!["--version".into()]),
                                                         AppMessage::CommandOutput,
            ),
            AppMessage::CommandOutput(output) => {
                self.tty_output.push_str(&output);
                self.tty_output.push('\n');
                Task::none()
            }
            AppMessage::ToggleTty => {
                self.tty_expanded = !self.tty_expanded;
                Task::none()
            }
            AppMessage::Profiles(msg) => {
                if let profiles::Message::Loaded(output) = &msg {
                    self.tty_output.push_str(output);
                    self.tty_output.push('\n');
                }
                self.profiles.update(&self.geode_path, msg).map(AppMessage::Profiles)
            }
            AppMessage::New(msg) => {
                if let new::Message::ProjectCreated(output) = &msg {
                    self.tty_output.push_str(output);
                    self.tty_output.push('\n');
                }
                self.new.update(&self.geode_path, msg).map(AppMessage::New)
            }
            AppMessage::Build(msg) => {
                if let build::Message::BuildLine(line) = &msg {
                    self.tty_output.push_str(line);
                    self.tty_output.push('\n');
                }
                self.build.update(&self.geode_path, msg).map(AppMessage::Build)
            }
        }
    }

    fn ttyview(&self) -> Element<AppMessage> {
        let arrow = if self.tty_expanded { "⌄" } else { ">" };
        let toggle = button(row![text("Terminal").size(14), text(arrow).size(12)].spacing(8))
        .on_press(AppMessage::ToggleTty)
        .padding(10)
        .width(Length::Fill)
        .style(togglestyle);
        let version_test = button(text("Version Test").size(20))
        .on_press(AppMessage::RunGeodeVersion)
        .padding([6, 12])
        .style(buttonstyle2);

        let header = row![toggle, version_test].spacing(6).align_y(iced::Alignment::Center);
        let mut panel = column![header].width(Length::Fill);

        if self.tty_expanded {
            let output = text(self.tty_output.as_str()).font(Font::MONOSPACE).size(13);
            panel = panel.push(
                container(scrollable(output).height(Length::Fixed(150.0)))
                .width(Length::Fill)
                .padding(15)
                .style(ttystyle),
            );
        }

        panel.into()
    }

    fn view(&self) -> Element<AppMessage> {
        let tab_bar = TabBar::new(AppMessage::TabSelected)
        .push(0, TabLabel::Text(String::from("Profiles")))
        .push(1, TabLabel::Text(String::from("New")))
        .push(2, TabLabel::Text(String::from("Build/Install")))
        .style(tabsstyle)
        .set_active_tab(&self.selected_tab);
        let content: Element<AppMessage> = match self.selected_tab {
            0 => self.profiles.view().map(AppMessage::Profiles),
            1 => self.new.view().map(AppMessage::New),
            2 => self.build.view().map(AppMessage::Build),
            _ => text("Unknown tab").into(),
        };
        let content_area = container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16);
        column![tab_bar, content_area, self.ttyview()]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
