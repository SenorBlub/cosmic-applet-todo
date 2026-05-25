use std::path::PathBuf;

use cosmic::app::{Core, Task};
use cosmic::iced::alignment::Vertical;
use cosmic::iced::window::Id;
use cosmic::iced::{Length, Rectangle};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::reorderable_flex_row::reorderable_flex_row;
use cosmic::widget::{
    button, container, divider, icon, scrollable, segmented_button, segmented_control, settings,
    text, text_input, toggler, Column, Row,
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

#[derive(Clone, Debug)]
struct EditState {
    horizon: Horizon,
    index: usize,
    buffer: String,
}

pub struct Window {
    core: Core,
    config: Config,
    popup: Option<Id>,
    todo: TodoFile,
    section_inputs: [String; 3],
    mode: Mode,
    horizon_model: segmented_button::SingleSelectModel,
    horizon_entities: [segmented_button::Entity; 3],
    editing: Option<EditState>,
    settings_time: String,
    settings_enabled: bool,
    settings_path: String,
    settings_feedback: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    Surface(cosmic::surface::Action),
    HorizonSelected(segmented_button::Entity),
    Toggle(Horizon, usize),
    Delete(Horizon, usize),
    SectionInput(Horizon, String),
    SectionSubmit(Horizon),
    BeginEdit(Horizon, usize),
    EditInput(String),
    CommitEdit,
    CancelEdit,
    Reorder(Horizon, Vec<usize>),
    OpenInEditor,
    EnterSettings,
    ExitSettings,
    SettingsTimeInput(String),
    SettingsEnabledToggle(bool),
    SettingsPathInput(String),
    SettingsSave,
}

fn build_horizon_model() -> (
    segmented_button::SingleSelectModel,
    [segmented_button::Entity; 3],
) {
    let mut model = segmented_button::SingleSelectModel::default();
    let today = model.insert().text("Today").id();
    let week = model.insert().text("This Week").id();
    let someday = model.insert().text("Someday").id();
    model.activate(today);
    (model, [today, week, someday])
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

    fn current_horizon(&self) -> Horizon {
        let active = self.horizon_model.active();
        for (i, entity) in self.horizon_entities.iter().enumerate() {
            if *entity == active {
                return Horizon::ALL[i];
            }
        }
        Horizon::Today
    }

    fn pending_today_count(&self) -> usize {
        self.todo
            .bucket(Horizon::Today)
            .iter()
            .filter(|t| !t.done)
            .count()
    }

    fn section_tasks(&self, h: Horizon) -> Element<'_, cosmic::Action<Message>> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let tasks = self.todo.bucket(h);

        if tasks.is_empty() {
            return container(text::caption("Nothing here yet"))
                .padding([spacing.space_xxxs, spacing.space_xs])
                .width(Length::Fill)
                .into();
        }

        let mut list = reorderable_flex_row(move |new_order: Vec<usize>| {
            cosmic::Action::App(Message::Reorder(h, new_order))
        })
        .spacing(spacing.space_xxs)
        .width(Length::Fill);

        let editing_index = self.editing.as_ref().and_then(|state| {
            if state.horizon == h {
                Some(state.index)
            } else {
                None
            }
        });

        for (i, task) in tasks.iter().enumerate() {
            let row = self.task_row(h, i, task);
            // Don't allow dragging the row that's currently being edited —
            // the user is typing into a text_input, drag would be confusing.
            list = if Some(i) == editing_index {
                list.push_locked(i, row)
            } else {
                list.push(i, row)
            };
        }

        list.into()
    }

    fn task_row(
        &self,
        h: Horizon,
        i: usize,
        task: &TodoTask,
    ) -> Element<'_, cosmic::Action<Message>> {
        let editing_this = matches!(
            &self.editing,
            Some(state) if state.horizon == h && state.index == i
        );
        if editing_this {
            self.task_edit_row()
        } else {
            self.task_view_row(h, i, task)
        }
    }

    fn task_view_row(
        &self,
        h: Horizon,
        i: usize,
        task: &TodoTask,
    ) -> Element<'_, cosmic::Action<Message>> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let icon_name: &str = if task.done {
            &self.config.icon_clear
        } else {
            &self.config.icon_pending
        };

        let toggle = button::icon(icon::from_name(icon_name.to_string()).size(16))
            .on_press(cosmic::Action::App(Message::Toggle(h, i)));

        let label = button::text(task.text.clone())
            .on_press(cosmic::Action::App(Message::BeginEdit(h, i)))
            .width(Length::Fill);

        let del = button::icon(icon::from_name("window-close-symbolic"))
            .on_press(cosmic::Action::App(Message::Delete(h, i)));

        Row::new()
            .spacing(spacing.space_xxs)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .push(toggle)
            .push(label)
            .push(del)
            .into()
    }

    fn task_edit_row(&self) -> Element<'_, cosmic::Action<Message>> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let buffer = self
            .editing
            .as_ref()
            .map(|s| s.buffer.clone())
            .unwrap_or_default();

        let input = text_input("Task…", buffer)
            .on_input(|s| cosmic::Action::App(Message::EditInput(s)))
            .on_submit(|_| cosmic::Action::App(Message::CommitEdit))
            .padding(spacing.space_xs)
            .width(Length::Fill);

        let cancel = button::icon(icon::from_name("window-close-symbolic"))
            .on_press(cosmic::Action::App(Message::CancelEdit));

        Row::new()
            .spacing(spacing.space_xxs)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .push(input)
            .push(cancel)
            .into()
    }

    fn section_input(&self, h: Horizon) -> Element<'_, cosmic::Action<Message>> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        let value = self.section_inputs[h.index()].clone();
        let placeholder = format!("Add to {}…", h.heading().to_lowercase());
        text_input(placeholder, value)
            .on_input(move |s| cosmic::Action::App(Message::SectionInput(h, s)))
            .on_submit(move |_| cosmic::Action::App(Message::SectionSubmit(h)))
            .padding(spacing.space_xs)
            .into()
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
        let horizon = self.current_horizon();

        let toggle = segmented_control::horizontal(&self.horizon_model)
            .width(Length::Fill)
            .on_activate(|e| cosmic::Action::App(Message::HorizonSelected(e)));

        // Only the task list scrolls; toggle + input + edit button stay docked.
        let scroll = scrollable(self.section_tasks(horizon)).height(Length::Fill);

        let col = Column::new()
            .spacing(spacing.space_s)
            .padding(spacing.space_s)
            .width(Length::Fixed(self.config.popup_width as f32))
            .height(Length::Fixed(self.config.popup_height as f32))
            .push(self.tasks_header())
            .push(toggle)
            .push(scroll)
            .push(self.section_input(horizon))
            .push(divider::horizontal::default())
            .push(
                button::standard("Edit todo file")
                    .on_press(cosmic::Action::App(Message::OpenInEditor))
                    .width(Length::Fill),
            );

        Element::from(self.core.applet.popup_container(col))
    }

    fn settings_view(&self) -> Element<'_, cosmic::Action<Message>> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        let time_input = text_input("HH:MM", self.settings_time.clone())
            .on_input(|s| cosmic::Action::App(Message::SettingsTimeInput(s)))
            .padding(spacing.space_xs)
            .width(Length::Fixed(96.0));

        let enabled_toggle = toggler(self.settings_enabled)
            .on_toggle(|v| cosmic::Action::App(Message::SettingsEnabledToggle(v)));

        let checkin_section = settings::section()
            .title("Daily check-in")
            .add(settings::item("Time (HH:MM)", time_input))
            .add(settings::item("Enabled", enabled_toggle));

        let path_input = text_input("~/todo.md", self.settings_path.clone())
            .on_input(|s| cosmic::Action::App(Message::SettingsPathInput(s)))
            .padding(spacing.space_xs)
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
        let (horizon_model, horizon_entities) = build_horizon_model();
        let window = Window {
            core,
            config: flags,
            popup: None,
            todo,
            section_inputs: Default::default(),
            mode: Mode::Tasks,
            horizon_model,
            horizon_entities,
            editing: None,
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
            Message::HorizonSelected(entity) => {
                self.horizon_model.activate(entity);
                self.editing = None;
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
                    if let Some(state) = &self.editing {
                        if state.horizon == h && state.index >= i {
                            self.editing = None;
                        }
                    }
                }
            }
            Message::BeginEdit(h, i) => {
                if let Some(task) = self.todo.bucket(h).get(i) {
                    self.editing = Some(EditState {
                        horizon: h,
                        index: i,
                        buffer: task.text.clone(),
                    });
                }
            }
            Message::EditInput(s) => {
                if let Some(state) = &mut self.editing {
                    state.buffer = s;
                }
            }
            Message::CommitEdit => {
                if let Some(state) = self.editing.take() {
                    let trimmed = state.buffer.trim().to_string();
                    if trimmed.is_empty() {
                        // Empty edits are rejected — restore the edit state.
                        self.editing = Some(state);
                    } else if let Some(task) =
                        self.todo.bucket_mut(state.horizon).get_mut(state.index)
                    {
                        if task.text != trimmed {
                            task.text = trimmed;
                            self.save_todo();
                        }
                    }
                }
            }
            Message::CancelEdit => {
                self.editing = None;
            }
            Message::Reorder(h, new_order) => {
                let bucket = self.todo.bucket_mut(h);
                let len = bucket.len();
                // Defend against malformed callback input — only accept a
                // permutation that references each current index exactly once.
                let mut seen = vec![false; len];
                let valid = new_order.len() == len
                    && new_order.iter().all(|&i| {
                        if i < len && !seen[i] {
                            seen[i] = true;
                            true
                        } else {
                            false
                        }
                    });
                if !valid {
                    log::warn!(
                        "Reorder ignored: invalid permutation for {} ({:?})",
                        h.heading(),
                        new_order
                    );
                } else {
                    let reordered: Vec<TodoTask> =
                        new_order.iter().map(|&i| bucket[i].clone()).collect();
                    *bucket = reordered;
                    // Cancel any in-flight edit on this horizon — the index
                    // it referred to no longer points at the same task.
                    if matches!(&self.editing, Some(s) if s.horizon == h) {
                        self.editing = None;
                    }
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
        let default_horizon_entity = self.horizon_entities[0];
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
                            state.horizon_model.activate(default_horizon_entity);
                            state.editing = None;
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
