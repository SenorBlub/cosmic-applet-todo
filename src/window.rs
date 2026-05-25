use std::path::PathBuf;

use cosmic::app::{Core, Task};
use cosmic::iced::alignment::Vertical;
use cosmic::iced::window::Id;
use cosmic::iced::{Length, Rectangle};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{
    button, container, divider, icon, scrollable, settings, text, text_input, toggler, Column, Row,
};
use cosmic::Element;

use crate::checkin;
use crate::config::Config;
use crate::storage::{self, Horizon, Task as TodoTask, TodoFile};

const ID: &str = "dev.thomasverhappen.CosmicAppletTodo";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Tasks,
    Settings,
}

pub struct Window {
    core: Core,
    config: Config,
    popup: Option<Id>,
    todo: TodoFile,
    section_inputs: [String; 3],
    mode: Mode,
    settings_time: String,
    settings_enabled: bool,
    settings_path: String,
    settings_feedback: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    Surface(cosmic::surface::Action),
    Toggle(Horizon, usize),
    Delete(Horizon, usize),
    SectionInput(Horizon, String),
    SectionSubmit(Horizon),
    OpenInEditor,
    EnterSettings,
    ExitSettings,
    SettingsTimeInput(String),
    SettingsEnabledToggle(bool),
    SettingsPathInput(String),
    SettingsSave,
}

impl Window {
    fn save_todo(&self) {
        if let Err(e) = storage::save(&self.config.file_path, &self.todo) {
            log::warn!("Failed to save {}: {}", self.config.file_path.display(), e);
        }
    }

    fn reload(&mut self) {
        self.todo = storage::load(&self.config.file_path);
    }

    fn sync_settings_from_config(&mut self) {
        self.settings_time = self.config.daily_checkin_time.clone();
        self.settings_enabled = self.config.daily_checkin_enabled;
        self.settings_path = self.config.file_path.display().to_string();
        self.settings_feedback = None;
    }

    fn pending_today_count(&self) -> usize {
        self.todo
            .bucket(Horizon::Today)
            .iter()
            .filter(|t| !t.done)
            .count()
    }

    fn section_view(&self, h: Horizon) -> Element<'_, cosmic::Action<Message>> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        let mut col = Column::new().spacing(spacing.space_xxs).width(Length::Fill);
        col = col.push(text::heading(h.heading()));

        let tasks = self.todo.bucket(h);
        if tasks.is_empty() {
            col = col.push(
                container(text::caption("Nothing here yet"))
                    .padding([spacing.space_xxxs, spacing.space_xs]),
            );
        } else {
            for (i, task) in tasks.iter().enumerate() {
                let icon_name: &str = if task.done {
                    &self.config.icon_clear
                } else {
                    &self.config.icon_pending
                };
                let row_content = Row::new()
                    .spacing(spacing.space_xs)
                    .align_y(Vertical::Center)
                    .push(icon::from_name(icon_name.to_string()).size(16))
                    .push(text(task.text.clone()).width(Length::Fill));

                let toggle = button::custom(row_content)
                    .on_press(cosmic::Action::App(Message::Toggle(h, i)))
                    .width(Length::Fill);

                let del = button::icon(icon::from_name("window-close-symbolic"))
                    .on_press(cosmic::Action::App(Message::Delete(h, i)));

                let line = Row::new()
                    .spacing(spacing.space_xxs)
                    .align_y(Vertical::Center)
                    .push(toggle)
                    .push(del);
                col = col.push(line);
            }
        }

        let value = self.section_inputs[h.index()].clone();
        let placeholder = format!("Add to {}…", h.heading().to_lowercase());
        let input = text_input(placeholder, value)
            .on_input(move |s| cosmic::Action::App(Message::SectionInput(h, s)))
            .on_submit(move |_| cosmic::Action::App(Message::SectionSubmit(h)))
            .padding(spacing.space_xxs);
        col = col.push(input);

        col.into()
    }

    fn tasks_header(&self) -> Element<'_, cosmic::Action<Message>> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        Row::new()
            .spacing(spacing.space_xs)
            .align_y(Vertical::Center)
            .push(text::title4("Todo").width(Length::Fill))
            .push(
                button::icon(icon::from_name("preferences-system-symbolic"))
                    .on_press(cosmic::Action::App(Message::EnterSettings)),
            )
            .into()
    }

    fn settings_header(&self) -> Element<'_, cosmic::Action<Message>> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        Row::new()
            .spacing(spacing.space_xs)
            .align_y(Vertical::Center)
            .push(
                button::icon(icon::from_name("go-previous-symbolic"))
                    .on_press(cosmic::Action::App(Message::ExitSettings)),
            )
            .push(text::title4("Settings").width(Length::Fill))
            .into()
    }

    fn tasks_view(&self) -> Element<'_, cosmic::Action<Message>> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        let mut col = Column::new()
            .spacing(spacing.space_s)
            .padding(spacing.space_s)
            .width(Length::Fixed(self.config.popup_width as f32))
            .push(self.tasks_header());

        for (idx, h) in Horizon::ALL.iter().copied().enumerate() {
            col = col.push(self.section_view(h));
            if idx + 1 < Horizon::ALL.len() {
                col = col.push(divider::horizontal::default());
            }
        }

        col = col.push(divider::horizontal::default());
        col = col.push(
            button::standard("Edit todo file")
                .on_press(cosmic::Action::App(Message::OpenInEditor))
                .width(Length::Fill),
        );

        let scroll = scrollable(col).height(Length::Shrink);
        Element::from(self.core.applet.popup_container(scroll))
    }

    fn settings_view(&self) -> Element<'_, cosmic::Action<Message>> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        let time_input = text_input("HH:MM", self.settings_time.clone())
            .on_input(|s| cosmic::Action::App(Message::SettingsTimeInput(s)))
            .padding(spacing.space_xxs)
            .width(Length::Fixed(96.0));

        let enabled_toggle = toggler(self.settings_enabled)
            .on_toggle(|v| cosmic::Action::App(Message::SettingsEnabledToggle(v)));

        let checkin_section = settings::section()
            .title("Daily check-in")
            .add(settings::item("Time (HH:MM)", time_input))
            .add(settings::item("Enabled", enabled_toggle));

        let path_input = text_input("~/todo.md", self.settings_path.clone())
            .on_input(|s| cosmic::Action::App(Message::SettingsPathInput(s)))
            .padding(spacing.space_xxs)
            .width(Length::Fill);

        let storage_section = settings::section()
            .title("Storage")
            .add(settings::item("Todo file", path_input));

        let save_btn = button::suggested("Save")
            .on_press(cosmic::Action::App(Message::SettingsSave))
            .width(Length::Fill);

        let mut col = Column::new()
            .spacing(spacing.space_s)
            .padding(spacing.space_s)
            .width(Length::Fixed(self.config.popup_width as f32))
            .push(self.settings_header())
            .push(checkin_section)
            .push(storage_section);

        if let Some(feedback) = &self.settings_feedback {
            col = col.push(text::caption(feedback.clone()));
        }

        col = col.push(save_btn);

        let scroll = scrollable(col).height(Length::Shrink);
        Element::from(self.core.applet.popup_container(scroll))
    }

    fn popup_view(&self) -> Element<'_, cosmic::Action<Message>> {
        match self.mode {
            Mode::Tasks => self.tasks_view(),
            Mode::Settings => self.settings_view(),
        }
    }
}

impl cosmic::Application for Window {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = Config;
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Message>) {
        let todo = storage::load(&flags.file_path);
        let settings_time = flags.daily_checkin_time.clone();
        let settings_enabled = flags.daily_checkin_enabled;
        let settings_path = flags.file_path.display().to_string();
        let window = Window {
            core,
            config: flags,
            popup: None,
            todo,
            section_inputs: Default::default(),
            mode: Mode::Tasks,
            settings_time,
            settings_enabled,
            settings_path,
            settings_feedback: None,
        };
        (window, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
            Message::Toggle(h, i) => {
                if let Some(t) = self.todo.bucket_mut(h).get_mut(i) {
                    t.done = !t.done;
                    self.save_todo();
                }
            }
            Message::Delete(h, i) => {
                let bucket = self.todo.bucket_mut(h);
                if i < bucket.len() {
                    bucket.remove(i);
                    self.save_todo();
                }
            }
            Message::SectionInput(h, s) => {
                self.section_inputs[h.index()] = s;
            }
            Message::SectionSubmit(h) => {
                let text = std::mem::take(&mut self.section_inputs[h.index()]);
                let text = text.trim().to_string();
                if !text.is_empty() {
                    self.todo.bucket_mut(h).push(TodoTask { text, done: false });
                    self.save_todo();
                }
            }
            Message::OpenInEditor => {
                let path = self.config.file_path.clone();
                if let Err(e) = std::process::Command::new("xdg-open").arg(&path).spawn() {
                    log::warn!("xdg-open {} failed: {}", path.display(), e);
                }
            }
            Message::EnterSettings => {
                self.sync_settings_from_config();
                self.mode = Mode::Settings;
            }
            Message::ExitSettings => {
                self.mode = Mode::Tasks;
                self.settings_feedback = None;
            }
            Message::SettingsTimeInput(s) => {
                self.settings_time = s;
            }
            Message::SettingsEnabledToggle(v) => {
                self.settings_enabled = v;
            }
            Message::SettingsPathInput(s) => {
                self.settings_path = s;
            }
            Message::SettingsSave => {
                let trimmed_time = self.settings_time.trim().to_string();
                if let Err(e) = checkin::parse_hhmm(&trimmed_time) {
                    self.settings_feedback = Some(format!("Invalid time: {e}"));
                    return Task::none();
                }
                let trimmed_path = self.settings_path.trim().to_string();
                if trimmed_path.is_empty() {
                    self.settings_feedback = Some("Todo file path cannot be empty".to_string());
                    return Task::none();
                }

                self.config.daily_checkin_time = trimmed_time;
                self.config.daily_checkin_enabled = self.settings_enabled;
                self.config.file_path = PathBuf::from(&trimmed_path);

                if let Err(e) = self.config.save() {
                    self.settings_feedback = Some(format!("Save failed: {e}"));
                    return Task::none();
                }

                if let Err(e) = checkin::apply(
                    &self.config.daily_checkin_time,
                    self.config.daily_checkin_enabled,
                ) {
                    self.settings_feedback =
                        Some(format!("Saved config, but systemd update failed: {e}"));
                    return Task::none();
                }

                self.reload();
                self.settings_feedback = Some("Saved".to_string());
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let have_popup = self.popup;
        let pending = self.pending_today_count();
        let icon_name: &str = if pending == 0 {
            &self.config.icon_clear
        } else {
            &self.config.icon_pending
        };
        let popup_size = (self.config.popup_width, self.config.popup_height);
        let btn = self
            .core
            .applet
            .icon_button(icon_name)
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = have_popup {
                    Message::Surface(destroy_popup(id))
                } else {
                    Message::Surface(app_popup::<Window>(
                        move |state: &mut Window| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            state.mode = Mode::Tasks;
                            state.sync_settings_from_config();
                            state.reload();
                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                Some(popup_size),
                                None,
                                None,
                            );
                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };
                            popup_settings
                        },
                        Some(Box::new(move |state: &Window| state.popup_view())),
                    ))
                }
            });

        let tooltip_text = if pending == 0 {
            "Todo — all clear".to_string()
        } else {
            format!("Todo — {pending} for today")
        };

        Element::from(self.core.applet.applet_tooltip::<Message>(
            btn,
            tooltip_text,
            self.popup.is_some(),
            Message::Surface,
            None,
        ))
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        text("").into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
