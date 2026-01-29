use iced::{Element, Task, Theme, Subscription, time, Length, window, Size};
use iced::widget::{column, container, text, button, center, progress_bar, row, text_input, checkbox, scrollable, horizontal_space};
use crate::theme;
use crate::model::{TodoList, TodoItem, Config};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
enum ViewMode {
    Full,
    Mini,
}

#[derive(Debug, Clone, PartialEq)]
enum TimerPhase {
    Focus,
    Break,
}

pub struct PomimiApp {
    // Timer
    duration: Duration,
    remaining: Duration,
    is_running: bool,
    timer_phase: TimerPhase,
    focus_duration: Duration,
    break_duration: Duration,

    // Todo
    todo_lists: Vec<TodoList>,
    active_list_index: usize,
    new_todo_input: String,
    new_list_input: String,
    is_creating_list: bool,

    // App State
    config: Config,
    view_mode: ViewMode,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Timer
    ToggleTimer,
    Tick,
    ResetTimer,
    SetDuration(Duration),

    // Todo
    AddTodo,
    UpdateNewTodoInput(String),
    ToggleTodo(usize, bool),
    DeleteTodo(usize),
    SwitchList(usize),
    UpdateNewListInput(String),
    CreateList,
    ToggleCreateListMode,

    // App
    ToggleMiniMode,
    ToggleRunInTerminal(bool),
}

impl PomimiApp {
    pub fn new() -> (Self, Task<Message>) {
        let focus_duration = Duration::from_secs(25 * 60);
        let break_duration = Duration::from_secs(5 * 60);

        // Load config
        let config = Config::load();
        let todo_lists = config.todo_lists.clone();

        (
            Self {
                duration: focus_duration,
                remaining: focus_duration,
                is_running: false,
                timer_phase: TimerPhase::Focus,
                focus_duration,
                break_duration,

                todo_lists,
                active_list_index: 0,
                new_todo_input: String::new(),
                new_list_input: String::new(),
                is_creating_list: false,
                config,
                view_mode: ViewMode::Full,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        let mins = self.remaining.as_secs() / 60;
        let secs = self.remaining.as_secs() % 60;
        let phase = match self.timer_phase {
            TimerPhase::Focus => "Focus",
            TimerPhase::Break => "Break",
        };
        format!("Pomimi ({}) - {:02}:{:02}", phase, mins, secs)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Timer Logic
            Message::ToggleTimer => {
                self.is_running = !self.is_running;
                Task::none()
            }
            Message::Tick => {
                if self.is_running {
                    if self.remaining.as_secs() > 0 {
                        self.remaining = self.remaining.saturating_sub(Duration::from_secs(1));
                    } else {
                        // Timer finished, switch phase
                        match self.timer_phase {
                            TimerPhase::Focus => {
                                self.timer_phase = TimerPhase::Break;
                                self.duration = self.break_duration;
                                self.remaining = self.break_duration;
                            }
                            TimerPhase::Break => {
                                self.timer_phase = TimerPhase::Focus;
                                self.duration = self.focus_duration;
                                self.remaining = self.focus_duration;
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::ResetTimer => {
                self.is_running = false;
                self.timer_phase = TimerPhase::Focus;
                self.duration = self.focus_duration;
                self.remaining = self.focus_duration;
                Task::none()
            }
            Message::SetDuration(d) => {
                self.focus_duration = d;
                let break_secs = (d.as_secs() / 5).max(60);
                self.break_duration = Duration::from_secs(break_secs);

                self.timer_phase = TimerPhase::Focus;
                self.duration = self.focus_duration;
                self.remaining = self.focus_duration;
                self.is_running = false;
                Task::none()
            }

            // Todo Logic
            Message::UpdateNewTodoInput(input) => {
                self.new_todo_input = input;
                Task::none()
            }
            Message::AddTodo => {
                if !self.new_todo_input.trim().is_empty() {
                    if let Some(list) = self.todo_lists.get_mut(self.active_list_index) {
                        let id = list.items.len() as u64;
                        list.items.push(TodoItem {
                            id,
                            text: self.new_todo_input.trim().to_string(),
                            completed: false,
                        });
                        self.new_todo_input.clear();

                        self.config.todo_lists = self.todo_lists.clone();
                        self.config.save();
                    }
                }
                Task::none()
            }
            Message::ToggleTodo(index, is_checked) => {
                if let Some(list) = self.todo_lists.get_mut(self.active_list_index) {
                    if let Some(item) = list.items.get_mut(index) {
                        item.completed = is_checked;

                        self.config.todo_lists = self.todo_lists.clone();
                        self.config.save();
                    }
                }
                Task::none()
            }
            Message::DeleteTodo(index) => {
                if let Some(list) = self.todo_lists.get_mut(self.active_list_index) {
                    if index < list.items.len() {
                        list.items.remove(index);

                        self.config.todo_lists = self.todo_lists.clone();
                        self.config.save();
                    }
                }
                Task::none()
            }
            Message::SwitchList(index) => {
                if index < self.todo_lists.len() {
                    self.active_list_index = index;
                }
                Task::none()
            }
            Message::UpdateNewListInput(input) => {
                self.new_list_input = input;
                Task::none()
            }
            Message::ToggleCreateListMode => {
                self.is_creating_list = !self.is_creating_list;
                self.new_list_input.clear();
                Task::none()
            }
            Message::CreateList => {
                 if !self.new_list_input.trim().is_empty() {
                     self.todo_lists.push(TodoList {
                         name: self.new_list_input.trim().to_string(),
                         items: Vec::new(),
                     });
                     self.active_list_index = self.todo_lists.len() - 1;
                     self.is_creating_list = false;
                     self.new_list_input.clear();

                     self.config.todo_lists = self.todo_lists.clone();
                     self.config.save();
                 }
                 Task::none()
            }

            // App Logic
            Message::ToggleMiniMode => {
                match self.view_mode {
                    ViewMode::Full => {
                        self.view_mode = ViewMode::Mini;
                        window::get_latest().and_then(|id| {
                            Task::batch(vec![
                                window::resize(id, Size::new(350.0, 320.0)),
                                window::change_level(id, window::Level::AlwaysOnTop)
                            ])
                        })
                    }
                    ViewMode::Mini => {
                        self.view_mode = ViewMode::Full;
                        window::get_latest().and_then(|id| {
                            Task::batch(vec![
                                window::resize(id, Size::new(800.0, 600.0)),
                                window::change_level(id, window::Level::Normal)
                            ])
                        })
                    }
                }
            }
            Message::ToggleRunInTerminal(val) => {
                self.config.cli_mode_default = val;
                self.config.save();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let timer_section = self.view_timer();

        if self.view_mode == ViewMode::Mini {
            let content = column![
                timer_section,
                button(text("Full").color(theme::ACCENT)).style(theme::button_secondary).on_press(Message::ToggleMiniMode)
            ]
            .spacing(10)
            .padding(10)
            .align_x(iced::Alignment::Center);

            container(center(content))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::container_default)
                .into()
        } else {
            let todo_section = self.view_todos();
            let settings_section = self.view_settings();

            let content = column![
                row![
                     text("POMIMI").size(20).color(theme::ACCENT),
                     horizontal_space(),
                     button(text("Mini").color(theme::ACCENT)).style(theme::button_secondary).on_press(Message::ToggleMiniMode)
                ].align_y(iced::Alignment::Center).width(Length::Fill),
                timer_section,
                horizontal_space(),
                todo_section,
                horizontal_space(),
                settings_section
            ]
            .spacing(40)
            .padding(20)
            .align_x(iced::Alignment::Center);

            container(center(content))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::container_default)
                .into()
        }
    }

    fn view_timer(&self) -> Element<'_, Message> {
        let mins = self.remaining.as_secs() / 60;
        let secs = self.remaining.as_secs() % 60;
        let time_str = format!("{:02}:{:02}", mins, secs);

        let progress = 1.0 - (self.remaining.as_secs_f32() / self.duration.as_secs_f32());

        let time_text_size = if self.view_mode == ViewMode::Mini { 40 } else { 80 };

        let phase_text = match self.timer_phase {
            TimerPhase::Focus => "Focus",
            TimerPhase::Break => "Break",
        };

        let display = column![
            text(phase_text).size(16).color(theme::TEXT_DIM),
            text(time_str).size(time_text_size).color(theme::PRIMARY),
            progress_bar(0.0..=1.0, progress)
                .style(theme::progress_bar_style)
                .height(10),
            row![
                button(text(if self.is_running { "PAUSE" } else { "START" }).color(theme::PRIMARY))
                    .style(theme::button_primary)
                    .on_press(Message::ToggleTimer),
                button(text("RESET").color(theme::ACCENT))
                    .style(theme::button_secondary)
                    .on_press(Message::ResetTimer),
            ]
            .spacing(20),
        ]
        .spacing(10)
        .align_x(iced::Alignment::Center);

        if self.view_mode == ViewMode::Full {
             column![
                 display,
                 row![
                     button(text("25m").color(theme::ACCENT)).on_press(Message::SetDuration(Duration::from_secs(25 * 60))).style(theme::button_secondary),
                     button(text("5m").color(theme::ACCENT)).on_press(Message::SetDuration(Duration::from_secs(5 * 60))).style(theme::button_secondary),
                     button(text("15m").color(theme::ACCENT)).on_press(Message::SetDuration(Duration::from_secs(15 * 60))).style(theme::button_secondary),
                ].spacing(10)
             ].spacing(20).align_x(iced::Alignment::Center).into()
        } else {
            display.into()
        }
    }

    fn view_todos(&self) -> Element<'_, Message> {
        let mut list_tabs = row![].spacing(10);
        for (i, list) in self.todo_lists.iter().enumerate() {
            let color = if i == self.active_list_index { theme::PRIMARY } else { theme::ACCENT };
            let mut btn = button(text(&list.name).size(14).color(color))
                .on_press(Message::SwitchList(i));

            if i == self.active_list_index {
                btn = btn.style(theme::button_primary);
            } else {
                btn = btn.style(theme::button_secondary);
            }
            list_tabs = list_tabs.push(btn);
        }

        list_tabs = list_tabs.push(
            button(text("+").color(theme::ACCENT)).on_press(Message::ToggleCreateListMode).style(theme::button_secondary)
        );

        let list_creation = if self.is_creating_list {
            row![
                text_input("New List Name", &self.new_list_input)
                    .on_input(Message::UpdateNewListInput)
                    .on_submit(Message::CreateList)
                    .padding(5)
                    .width(Length::Fixed(150.0)),
                button(text("Add").color(theme::PRIMARY)).on_press(Message::CreateList).style(theme::button_primary)
            ].spacing(10)
        } else {
            row![].into()
        };

        let current_list = &self.todo_lists[self.active_list_index];
        let items: Element<Message> = if current_list.items.is_empty() {
             text("No tasks yet. Stay focused!").size(16).color(theme::TEXT_DIM).into()
        } else {
            let list_col = column(
                current_list.items.iter().enumerate().map(|(i, item)| {
                    row![
                        checkbox("", item.completed)
                            .on_toggle(move |checked| Message::ToggleTodo(i, checked)),
                        text(&item.text).size(18).color(if item.completed { theme::TEXT_DIM } else { theme::TEXT }),
                        button(text("x").color(theme::ACCENT)).on_press(Message::DeleteTodo(i)).style(theme::button_secondary)
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center)
                    .into()
                })
            ).spacing(10);

            scrollable(list_col).height(Length::Fixed(200.0)).into()
        };

        let add_todo_row = row![
            text_input("Add a new task...", &self.new_todo_input)
                .on_input(Message::UpdateNewTodoInput)
                .on_submit(Message::AddTodo)
                .padding(10),
            button(text("Add").color(theme::PRIMARY)).on_press(Message::AddTodo).style(theme::button_primary)
        ].spacing(10);

        column![
            list_tabs,
            list_creation,
            container(items).padding(10).style(theme::container_bordered),
            add_todo_row
        ]
        .spacing(15)
        .width(Length::Fixed(400.0))
        .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        row![
            checkbox("Run in Terminal by Default", self.config.cli_mode_default)
                .on_toggle(Message::ToggleRunInTerminal),
        ]
        .spacing(20)
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.is_running {
            time::every(Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}
