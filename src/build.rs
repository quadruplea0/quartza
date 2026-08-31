use crate::{buttonstyle1, buttonstyle2, textboxstyle};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length, Task};
use std::f64::consts::E;
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use iced::futures::SinkExt;

#[derive(Debug, Clone)]
enum BuildEvent {
    Line(String),
    Done,
}

fn spawn_reader(read: impl Read + Send + 'static, tx: mpsc::Sender<String>) {
    std::thread::spawn(move || {
        let reader = BufReader::new(read);
        for line in reader.lines().flatten() {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
}

fn build_stream(geode_path: String, project_dir: String) -> impl iced::futures::Stream<Item = BuildEvent> {
    iced::stream::channel(100, |mut sender: iced::futures::channel::mpsc::Sender<BuildEvent>| async move {
        let mut child = match Command::new(&geode_path)
        .arg("build")
        .current_dir(&project_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        {
            Ok(child) => child,
                          Err(e) => {
                              let _ = sender.send(BuildEvent::Line(format!("build failed: {e}"))).await;
                              let _ = sender.send(BuildEvent::Done).await;
                              return;
                          }
        };

        let (tx, rx) = mpsc::channel();
        if let Some(stdout) = child.stdout.take() {
            spawn_reader(stdout, tx.clone());
        }
        if let Some(stderr) = child.stderr.take() {
            spawn_reader(stderr, tx.clone());
        }
        drop(tx);

        while let Ok(line) = rx.recv() {
            if sender.send(BuildEvent::Line(line)).await.is_err() {
                break;
            }
        }

        let _ = child.wait();
        let _ = sender.send(BuildEvent::Done).await;
    })
}

async fn pick_project_dir() -> Option<String> {
    rfd::AsyncFileDialog::new()
    .pick_folder()
    .await
    .map(|handle| handle.path().to_string_lossy().to_string())
}

#[derive(Debug, Clone)]
pub enum Message {
    ProjectDirChanged(String),
    BrowseProjectDir,
    ProjectDirPicked(Option<String>),
    StartBuild,
    BuildLine(String),
    BuildFinished,
}

#[derive(Default)]
pub struct Tab {
    project_directory: String,
    building: bool,
    latest_line: String,
}

impl Tab {
    pub fn update(&mut self, geode_path: &str, message: Message) -> Task<Message> {
        match message {
            Message::ProjectDirChanged(v) => {
                self.project_directory = v;
                Task::none()
            }
            Message::BrowseProjectDir => Task::perform(pick_project_dir(), Message::ProjectDirPicked),
            Message::ProjectDirPicked(path) => {
                if let Some(path) = path {
                    self.project_directory = path;
                }
                Task::none()
            }
            Message::StartBuild => {
                if self.project_directory.is_empty() {
                    return Task::none();
                }
                self.building = true;
                self.latest_line.clear();
                Task::run(
                    build_stream(geode_path.to_string(), self.project_directory.clone()),
                          |event| match event {
                              BuildEvent::Line(line) => Message::BuildLine(line),
                          BuildEvent::Done => Message::BuildFinished,
                          },
                )
            }
            Message::BuildLine(line) => {
                self.latest_line = line;
                Task::none()
            }
            Message::BuildFinished => {
                self.building = false;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let dir_row = row![
            text_input("Project folder", &self.project_directory)
            .on_input(Message::ProjectDirChanged)
            .padding(10)
            .width(Length::Fill)
            .style(textboxstyle),
            button(text("Browse").size(16))
            .on_press(Message::BrowseProjectDir)
            .padding([10, 16])
            .style(buttonstyle2),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        let build_label = if self.building { "Building project, please wait" } else { "Build" };
        let build_button = button(text(build_label).size(14))
        .padding([10, 20])
        .style(buttonstyle1);
        let build_button: Element<Message> = if self.building || self.project_directory.is_empty() {
            build_button.into()
        } else {
            build_button.on_press(Message::StartBuild).into()
        };

        let status_line: Element<Message> = if self.latest_line.is_empty() {
            text("").size(13).into()
        } else {
            container(text(self.latest_line.clone()).size(13))
            .width(Length::Fill)
            .padding(8)
            .into()
        };

        column![
            text("Build/Install").size(20),
            column![text("Project folder").size(13), dir_row].spacing(6),
            build_button,
            status_line,
        ]
        .spacing(14)
        .into()
    }
}
