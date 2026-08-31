use crate::{buttonstyle1, buttonstyle2, textboxstyle};
use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input};
use iced::{Element, Length, Task};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    Default,
    Minimal,
}

impl Template {
    const ALL: [Template; 2] = [Template::Default, Template::Minimal];
    fn down_presses(self) -> usize {
        match self {
            Template::Default => 0,
            Template::Minimal => 1,
        }
    }
}

impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Template::Default => "Default",
            Template::Minimal => "Minimal",
        };
        write!(f, "{label}")
    }
}

fn wait_for(buf: &Arc<Mutex<String>>, needle: &str, timeout: Duration) -> bool {
    let start = Instant::now();
    loop {
        if buf.lock().unwrap().contains(needle) {
            return true;
        }
        if start.elapsed() > timeout {
            return false;
        }
        std::thread::sleep(Duration::from_millis(30));
    }
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                chars.next();
            }
        } else {
            out.push(c);
        }
    }
    out
}

async fn pick_target_dir() -> Option<String> {
    rfd::AsyncFileDialog::new()
    .set_title("Project folder")
    .pick_folder()
    .await
    .map(|handle| handle.path().to_string_lossy().to_string())
}

async fn run_geode_new(
    geode_path: String,
    mod_name: String,
    version: String,
    developer: String,
    description: String,
    target_directory: String,
    strip_comments: bool,
    template: Template,
) -> String {
    let pty_system = native_pty_system();
    let pair = match pty_system.openpty(PtySize { rows: 40, cols: 120, pixel_width: 0, pixel_height: 0 }) {
        Ok(pair) => pair,
        Err(e) => return format!("opening PTY failed: {e}"),
    };

    let mut cmd = CommandBuilder::new(geode_path);
    cmd.arg("new");
    cmd.env("TERM", "xterm-256color");

    let mut child = match pair.slave.spawn_command(cmd) {
        Ok(child) => child,
        Err(e) => return format!("geode failed: {e}"),
    };
    drop(pair.slave);

    let mut reader = match pair.master.try_clone_reader() {
        Ok(r) => r,
        Err(e) => return format!("cloning PTY reader fialed: {e}"),
    };
    let mut writer = match pair.master.take_writer() {
        Ok(w) => w,
        Err(e) => return format!("taking PTY writer failed: {e}"),
    };

    let output = Arc::new(Mutex::new(String::new()));
    let output_reader = output.clone();
    std::thread::spawn(move || {
        let mut chunk = [0u8; 4096];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                       Ok(n) => output_reader.lock().unwrap().push_str(&String::from_utf8_lossy(&chunk[..n])),
                       Err(_) => break,
            }
        }
    });

    macro_rules! step {
        ($needle:expr, $answer:expr) => {
            if !wait_for(&output, $needle, Duration::from_secs(6)) {
                let _ = child.kill();
                let captured = strip_ansi(&output.lock().unwrap());
                return format!(
                    "{captured}\n\n[Quartz] timed out waiting for '{}'.",
                    $needle
                );
            }
            std::thread::sleep(Duration::from_millis(80));
            let _ = writer.write_all(&[0x7f; 200]);
            let _ = writer.write_all($answer.as_bytes());
        };
    }
    std::thread::sleep(Duration::from_millis(150));
    for _ in 0..template.down_presses() {
        let _ = writer.write_all(b"\x1b[B");
        std::thread::sleep(Duration::from_millis(60));
    }
    let _ = writer.write_all(b"\r");

    step!("Name:", format!("{mod_name}\n"));
    step!("Version:", format!("{version}\n"));
    step!("Developer:", format!("{developer}\n"));
    step!("Description", format!("{description}\n"));
    step!("Location:", format!("{target_directory}\n"));

    if !wait_for(&output, "remove comments", Duration::from_secs(6)) {
        let _ = child.kill();
        let captured = strip_ansi(&output.lock().unwrap());
        return format!("{captured}\n\n[Quartz] timed out waiting for the strip comments prompt");
    }
    let answer = if strip_comments { "y\n" } else { "n\n" };
    let _ = writer.write_all(answer.as_bytes());

    if wait_for(&output, "Are you sure you want to proceed", Duration::from_secs(3)) {
        let _ = writer.write_all(b"y\n");
    }

    let (done_tx, done_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = done_tx.send(());
    });
    if done_rx.recv_timeout(Duration::from_secs(15)).is_err() {
        let captured = strip_ansi(&output.lock().unwrap());
        return format!("{captured}\n\n[Quartz] geode did not exit after all answers were sent.");
    }

    std::thread::sleep(Duration::from_millis(100));
    strip_ansi(&output.lock().unwrap())
}

#[derive(Debug, Clone)]
pub enum Message {
    ModNameChanged(String),
    VersionChanged(String),
    DeveloperChanged(String),
    DescriptionChanged(String),
    TargetDirChanged(String),
    BrowseTargetDir,
    TargetDirPicked(Option<String>),
    StripCommentsToggled(bool),
    TemplateSelected(Template),
    CreateProject,
    ProjectCreated(String),
}

#[derive(Default)]
pub struct Tab {
    mod_name: String,
    version: String,
    developer: String,
    description: String,
    target_directory: String,
    strip_comments: bool,
    template: Option<Template>,
    creating: bool,
}

impl Tab {
    pub fn update(&mut self, geode_path: &str, message: Message) -> Task<Message> {
        match message {
            Message::ModNameChanged(v) => { self.mod_name = v; Task::none() }
            Message::VersionChanged(v) => { self.version = v; Task::none() }
            Message::DeveloperChanged(v) => { self.developer = v; Task::none() }
            Message::DescriptionChanged(v) => { self.description = v; Task::none() }
            Message::TargetDirChanged(v) => { self.target_directory = v; Task::none() }
            Message::BrowseTargetDir => Task::perform(pick_target_dir(), Message::TargetDirPicked),
            Message::TargetDirPicked(path) => {
                if let Some(path) = path {
                    self.target_directory = path;
                }
                Task::none()
            }
            Message::StripCommentsToggled(v) => { self.strip_comments = v; Task::none() }
            Message::TemplateSelected(t) => { self.template = Some(t); Task::none() }
            Message::CreateProject => {
                let Some(template) = self.template else { return Task::none() };
                self.creating = true;
                Task::perform(
                    run_geode_new(
                        geode_path.to_string(),
                                  self.mod_name.clone(),
                                  self.version.clone(),
                                  self.developer.clone(),
                                  self.description.clone(),
                                  self.target_directory.clone(),
                                  self.strip_comments,
                                  template,
                    ),
                    Message::ProjectCreated,
                )
            }
            Message::ProjectCreated(_output) => {
                self.creating = false;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let target_dir_row = row![
            text_input("Project location", &self.target_directory)
            .on_input(Message::TargetDirChanged)
            .padding(10)
            .width(Length::Fill)
            .style(textboxstyle),
            button(text("Browse").size(13))
            .on_press(Message::BrowseTargetDir)
            .padding([10, 16])
            .style(buttonstyle2),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        let create_label = if self.creating { "Creating project, please wait" } else { "Create Project" };
        let create_button = button(text(create_label).size(14))
        .padding([10, 20])
        .style(buttonstyle1);
        let create_button: Element<Message> = if self.creating || self.template.is_none() {
            create_button.into()
        } else {
            create_button.on_press(Message::CreateProject).into()
        };

        column![
            text("New Mod").size(20),
            field("Mod name", &self.mod_name, "Mod Name", Message::ModNameChanged),
            field("Version", &self.version, "v1.0.0", Message::VersionChanged),
            field("Developer", &self.developer, "Your name", Message::DeveloperChanged),
            field("Description", &self.description, "What does the mod do?", Message::DescriptionChanged),
            column![text("Target directory").size(13), target_dir_row].spacing(6),
            column![
                text("Template").size(13),
                pick_list(Template::ALL, self.template, Message::TemplateSelected)
                .placeholder("Choose a template"),
            ]
            .spacing(6),
            checkbox(self.strip_comments)
            .label("Strip comments from default template")
            .on_toggle(Message::StripCommentsToggled),
            create_button,
            container(text("")).height(Length::Fixed(20.0)),
        ]
        .spacing(14)
        .into()
    }
}

fn field<'a>(
    label: &'a str,
    value: &'a str,
    placeholder: &'a str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    column![
        text(label).size(13),
        text_input(placeholder, value).on_input(on_change).padding(10).style(textboxstyle),
    ]
    .spacing(6)
    .into()
}
