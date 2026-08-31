use crate::{panelstyle, run_command, buttonstyle2};
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Task};

#[derive(Debug, Clone)]
struct Profile {
    name: String,
    path: String,
    active: bool,
}

fn parse_profiles(output: &str) -> Vec<Profile> {
    output
    .lines()
    .filter_map(|line| {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }
        let active = line.starts_with('*');
        let line = line.trim_start_matches('*').trim();
        let open = line.find('[')?;
        let close = line.rfind(']')?;
        let name = line[..open].trim().to_string();
        let inner = line[open + 1..close].trim();
        let path = inner
        .strip_prefix("path")?
        .trim_start()
        .trim_start_matches('=')
        .trim()
        .to_string();
        Some(Profile { name, path, active })
    })
    .collect()
}

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    Loaded(String),
    Select(String),
    Selected(String),
}

#[derive(Default)]
pub struct Tab {
    profiles: Vec<Profile>,
}

impl Tab {
    pub fn update(&mut self, geode_path: &str, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => Task::perform(
                run_command(geode_path.to_string(), vec!["profile".into(), "list".into()]),
                                              Message::Loaded,
            ),
            Message::Loaded(output) => {
                self.profiles = parse_profiles(&output);
                Task::none()
            }
            Message::Select(name) => Task::perform(
                run_command(
                    geode_path.to_string(),
                            vec!["profile".into(), "select".into(), name.clone()],
                ),
                move |_| Message::Selected(name.clone()),
            ),
            Message::Selected(name) => {
                for profile in &mut self.profiles {
                    profile.active = profile.name == name;
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut list = column![].spacing(8);
        for profile in &self.profiles {
            let label = if profile.active {
                format!("* {}", profile.name)
            } else {
                format!(" {}", profile.name)
            };
            list = list.push(
                container(
                    row![
                        column![text(label).size(15), text(profile.path.clone()).size(12)]
                        .spacing(2)
                        .width(Length::Fill),
                          button(text("Select").size(15))
                          .on_press(Message::Select(profile.name.clone()))
                          .padding([6, 14])
                          .style(buttonstyle2),
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center),
                )
                .padding(12)
                .width(Length::Fill)
                .style(panelstyle),
            );
        }
        column![
            text("Profiles").size(20),
            row![
                button(text("Refresh").size(13))
                .on_press(Message::Refresh)
                .padding([8, 16])
                .style(buttonstyle2),
            ]
            .spacing(10),
            list,
        ]
        .spacing(14)
        .into()
    }
}
