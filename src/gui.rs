use iced::{Element, Task, Theme, Subscription, time, Length, window, Size, Color};
use iced::widget::{column, container, text, button, center, row, text_input, scrollable, horizontal_space, stack};
use crate::theme;
use crate::model::{Database, Task as DbTask};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Full,
    Mini,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Phase {
    fn duration_secs(&self) -> u64 {
        match self {
            Phase::Focus => 25 * 60,
            Phase::ShortBreak => 5 * 60,
            Phase::LongBreak => 30 * 60,
        }
    }

    fn label(&self) -> &str {
        match self {
            Phase::Focus => "Pomodoro Active",
            Phase::ShortBreak => "Short Break",
            Phase::LongBreak => "Long Break",
        }
    }
}

#[derive(Debug, Clone)]
struct TimerState {
    phase: Phase,
    remaining_secs: u64,
    total_secs: u64,
    is_running: bool,
    cycles_completed: usize,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            phase: Phase::Focus,
            remaining_secs: Phase::Focus.duration_secs(),
            total_secs: Phase::Focus.duration_secs(),
            is_running: false,
            cycles_completed: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct State {
    db: Database,
    tasks: Vec<DbTask>,
    timer: TimerState,
    session_focus_seconds: i64,
    view_mode: ViewMode,
    new_task_input: String,
    active_task_id: Option<i64>,
    settings_open: bool,
    primary_color: Color,
    is_dark_mode: bool,
}

pub enum PomimiApp {
    Loading,
    Loaded(State),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    // Initialization
    FontLoaded(Result<(), iced::font::Error>),
    DbConnected(Result<Database, String>),
    TasksLoaded(Result<Vec<DbTask>, String>),
    SessionLoaded(Result<i64, String>),
    TaskOperationFailed(String),
    TaskOperationSuccess,

    // Timer
    ToggleTimer,
    Tick,
    ResetTimer,
    SkipPhase,

    // Tasks
    UpdateNewTaskInput(String),
    AddTask,
    DeleteTask(i64),
    MarkTaskDone(i64),
    SetActiveTask(i64),

    // UI
    ToggleMiniMode,
    ToggleSettings,
    SetColor(Color),
    ToggleTheme,

    None,
}

impl PomimiApp {
    pub fn new() -> (Self, Task<Message>) {
        // Load fonts
        let fonts = Task::batch(vec![
            iced::font::load(std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/SpaceGrotesk-Regular.ttf").as_slice())).map(Message::FontLoaded),
            iced::font::load(std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/SpaceGrotesk-Bold.ttf").as_slice())).map(Message::FontLoaded),
            iced::font::load(std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/NotoSans-Regular.ttf").as_slice())).map(Message::FontLoaded),
            iced::font::load(std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/NotoSans-Bold.ttf").as_slice())).map(Message::FontLoaded),
            iced::font::load(std::borrow::Cow::Borrowed(include_bytes!("../assets/fonts/MaterialSymbolsOutlined.ttf").as_slice())).map(Message::FontLoaded),
        ]);

        let connect_db = Task::perform(
            async {
                Database::new().await.map_err(|e| e.to_string())
            },
            Message::DbConnected
        );

        (
            PomimiApp::Loading,
            Task::batch(vec![fonts, connect_db]),
        )
    }

    pub fn title(&self) -> String {
        match self {
            PomimiApp::Loading => "Pomimi".to_string(),
            PomimiApp::Error(_) => "Pomimi - Error".to_string(),
            PomimiApp::Loaded(state) => {
                let mins = state.timer.remaining_secs / 60;
                let secs = state.timer.remaining_secs % 60;
                format!("Pomimi - {:02}:{:02}", mins, secs)
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match self {
            PomimiApp::Loading => {
                match message {
                    Message::DbConnected(Ok(db)) => {
                        let load_tasks = Task::perform(
                            {
                                let db = db.clone();
                                async move { db.get_tasks().await.map_err(|e| e.to_string()) }
                            },
                            Message::TasksLoaded
                        );
                        let load_session = Task::perform(
                             {
                                let db = db.clone();
                                async move { db.get_today_focus_time().await.map_err(|e| e.to_string()) }
                             },
                             Message::SessionLoaded
                        );

                        *self = PomimiApp::Loaded(State {
                            db,
                            tasks: Vec::new(),
                            timer: TimerState::default(),
                            session_focus_seconds: 0,
                            view_mode: ViewMode::Full,
                            new_task_input: String::new(),
                            active_task_id: None,
                            settings_open: false,
                            primary_color: theme::ORANGE, // Default, TODO: Load from DB
                            is_dark_mode: true,
                        });

                        Task::batch(vec![load_tasks, load_session])
                    }
                    Message::DbConnected(Err(e)) => {
                        *self = PomimiApp::Error(format!("Failed to connect to database: {}", e));
                        Task::none()
                    }
                    Message::FontLoaded(_) => Task::none(),
                    _ => Task::none(),
                }
            }
            PomimiApp::Error(_) => Task::none(),
            PomimiApp::Loaded(state) => {
                match message {
                    Message::TasksLoaded(Ok(tasks)) => {
                        state.tasks = tasks;
                        // Auto-select first task if none active?
                        if state.active_task_id.is_none() && !state.tasks.is_empty() {
                            state.active_task_id = Some(state.tasks[0].id);
                        }
                        Task::none()
                    }
                    Message::TasksLoaded(Err(e)) => {
                        eprintln!("Failed to load tasks: {}", e);
                        Task::none()
                    }
                    Message::SessionLoaded(Ok(secs)) => {
                        state.session_focus_seconds = secs;
                        Task::none()
                    }
                    Message::SessionLoaded(Err(e)) => {
                         eprintln!("Failed to load session: {}", e);
                         Task::none()
                    }
                    Message::TaskOperationFailed(e) => {
                        eprintln!("Task operation failed: {}", e);
                        Task::none()
                    }
                    Message::TaskOperationSuccess => {
                         let db = state.db.clone();
                         Task::perform(
                            async move { db.get_tasks().await.map_err(|e| e.to_string()) },
                            Message::TasksLoaded
                        )
                    }

                    // Timer
                    Message::ToggleTimer => {
                        state.timer.is_running = !state.timer.is_running;
                        Task::none()
                    }
                    Message::Tick => {
                        if state.timer.is_running {
                            if state.timer.remaining_secs > 0 {
                                state.timer.remaining_secs -= 1;
                                if state.timer.phase == Phase::Focus {
                                    state.session_focus_seconds += 1;
                                }
                            } else {
                                let completed_phase = state.timer.phase.clone();
                                match completed_phase {
                                    Phase::Focus => {
                                        state.timer.cycles_completed += 1;
                                        let db = state.db.clone();
                                        let duration = completed_phase.duration_secs() as i64;
                                        let _ = Task::perform(
                                            async move { db.add_session(duration).await },
                                            |_| Message::None
                                        );

                                        if state.timer.cycles_completed % 4 == 0 {
                                            state.timer.phase = Phase::LongBreak;
                                        } else {
                                            state.timer.phase = Phase::ShortBreak;
                                        }

                                        // Auto-complete active task? No, user explicitly marks done.
                                    }
                                    Phase::ShortBreak | Phase::LongBreak => {
                                        state.timer.phase = Phase::Focus;
                                    }
                                }
                                state.timer.remaining_secs = state.timer.phase.duration_secs();
                                state.timer.total_secs = state.timer.phase.duration_secs();
                            }
                        }
                        Task::none()
                    }
                    Message::ResetTimer => {
                        state.timer.is_running = false;
                        state.timer.phase = Phase::Focus;
                        state.timer.remaining_secs = Phase::Focus.duration_secs();
                        state.timer.total_secs = Phase::Focus.duration_secs();
                        Task::none()
                    }
                    Message::SkipPhase => {
                         state.timer.remaining_secs = 0;
                         Task::none()
                    }

                    // Tasks
                    Message::UpdateNewTaskInput(input) => {
                        state.new_task_input = input;
                        Task::none()
                    }
                    Message::AddTask => {
                        if !state.new_task_input.trim().is_empty() {
                            let text = state.new_task_input.trim().to_string();
                            state.new_task_input.clear();
                            let db = state.db.clone();
                            Task::perform(
                                async move { db.add_task(&text).await.map_err(|e| e.to_string()) },
                                |res| match res {
                                    Ok(_) => Message::TaskOperationSuccess,
                                    Err(e) => Message::TaskOperationFailed(e),
                                }
                            )
                        } else {
                            Task::none()
                        }
                    }
                    Message::DeleteTask(id) => {
                        if state.active_task_id == Some(id) {
                            state.active_task_id = None;
                        }
                        let db = state.db.clone();
                        Task::perform(
                            async move { db.delete_task(id).await.map_err(|e| e.to_string()) },
                            |res| match res {
                                Ok(_) => Message::TaskOperationSuccess,
                                Err(e) => Message::TaskOperationFailed(e),
                            }
                        )
                    }
                    Message::MarkTaskDone(id) => {
                        // Mark done = delete per user request
                        if state.active_task_id == Some(id) {
                            state.active_task_id = None;
                        }
                         let db = state.db.clone();
                        Task::perform(
                            async move { db.delete_task(id).await.map_err(|e| e.to_string()) },
                             |res| match res {
                                Ok(_) => Message::TaskOperationSuccess,
                                Err(e) => Message::TaskOperationFailed(e),
                            }
                        )
                    }
                    Message::SetActiveTask(id) => {
                        state.active_task_id = Some(id);
                        Task::none()
                    }

                    // UI
                    Message::ToggleMiniMode => {
                        match state.view_mode {
                            ViewMode::Full => {
                                state.view_mode = ViewMode::Mini;
                                window::get_latest().and_then(|id| {
                                    Task::batch(vec![
                                        window::resize(id, Size::new(350.0, 320.0)),
                                        window::change_level(id, window::Level::AlwaysOnTop)
                                    ])
                                })
                            }
                            ViewMode::Mini => {
                                state.view_mode = ViewMode::Full;
                                window::get_latest().and_then(|id| {
                                    Task::batch(vec![
                                        window::resize(id, Size::new(800.0, 600.0)),
                                        window::change_level(id, window::Level::Normal)
                                    ])
                                })
                            }
                        }
                    }
                    Message::ToggleSettings => {
                        state.settings_open = !state.settings_open;
                        Task::none()
                    }
                    Message::SetColor(color) => {
                        state.primary_color = color;
                        // TODO: Persist
                        Task::none()
                    }
                    Message::ToggleTheme => {
                        state.is_dark_mode = !state.is_dark_mode;
                        Task::none()
                    }

                    _ => Task::none(),
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self {
            PomimiApp::Loading => center(text("Loading...").size(30)).into(),
            PomimiApp::Error(e) => center(text(format!("Error: {}", e)).size(20).color(Color::from_rgb(1.0, 0.0, 0.0))).into(),
            PomimiApp::Loaded(state) => {
                let timer_view = self.view_timer(state);

                // Background Text "FOCUS"
                let background_text = container(
                    text("FOCUS")
                        .size(150)
                        .font(iced::Font { family: iced::font::Family::Name("Space Grotesk"), weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT })
                        .color(Color { a: 0.05, ..theme::WHITE })
                ).align_x(iced::Alignment::Center);

                if state.view_mode == ViewMode::Mini {
                    let active_task_view: Element<'_, Message> = if let Some(id) = state.active_task_id {
                        if let Some(task) = state.tasks.iter().find(|t| t.id == id) {
                             container(
                                 row![
                                     container(horizontal_space().width(6).height(6))
                                         .style(|_t: &Theme| container::Style { background: Some(state.primary_color.into()), ..container::Style::default() }),
                                     column![
                                         text(&task.text).size(12).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT }),
                                     ]
                                 ].spacing(10).align_y(iced::Alignment::Center)
                             )
                             .padding(10)
                             .style(|_t: &Theme| container::Style { background: Some(Color{a:0.05, ..theme::WHITE}.into()), ..container::Style::default() })
                             .width(Length::Fill)
                             .into()
                        } else {
                            horizontal_space().into()
                        }
                    } else {
                        horizontal_space().into()
                    };

                    let content = column![
                         timer_view,
                         active_task_view,
                         button(text("Expand").size(10)).on_press(Message::ToggleMiniMode).style(theme::button_ghost)
                    ]
                    .align_x(iced::Alignment::Center)
                    .spacing(10)
                    .padding(10);

                    stack![
                         container(background_text).width(Length::Fill).height(Length::Fill).align_y(iced::Alignment::Center).align_x(iced::Alignment::Center),
                         container(content).width(Length::Fill).height(Length::Fill).style(theme::container_default)
                    ].into()

                } else {
                    let tasks_view = self.view_tasks(state);
                    let footer = self.view_footer(state);

                    let main_content = column![
                        timer_view,
                        horizontal_space().height(20),
                        tasks_view,
                        horizontal_space().height(Length::Fill),
                        footer
                    ]
                    .padding(40)
                    .max_width(500)
                    .align_x(iced::Alignment::Center);

                    stack![
                        container(background_text).width(Length::Fill).height(Length::Fill).padding(20).align_x(iced::Alignment::Center),
                        container(center(main_content))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .style(theme::container_default)
                    ].into()
                }
            }
        }
    }

    fn view_timer<'a>(&self, state: &'a State) -> Element<'a, Message> {
        let mins = state.timer.remaining_secs / 60;
        let secs = state.timer.remaining_secs % 60;
        let time_str = format!("{:02}:{:02}", mins, secs);

        let label = state.timer.phase.label();

        let mut col = column![
            text(time_str)
                .size(if state.view_mode == ViewMode::Mini { 60 } else { 100 })
                .font(iced::Font { family: iced::font::Family::Name("Space Grotesk"), weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT })
                .line_height(0.9),

            row![
                container(horizontal_space().width(8).height(8))
                    .style(|_t: &Theme| container::Style {
                        background: Some(state.primary_color.into()),
                        border: iced::Border { radius: 4.0.into(), ..iced::Border::default() },
                        ..container::Style::default()
                    }),
                text(label).size(12).font(iced::Font { weight: iced::font::Weight::Medium, ..iced::Font::DEFAULT }).color(theme::TEXT_DIM)
            ].spacing(8).align_y(iced::Alignment::Center)
        ].align_x(iced::Alignment::Center);

        if state.view_mode == ViewMode::Full {
             col = col.push(horizontal_space().height(20));
             col = col.push(
                 button(
                     row![
                         text(if state.timer.is_running { "PAUSE FOCUS" } else { "START FOCUS" }).size(14).font(iced::Font::MONOSPACE).color(Color::BLACK),
                         text("->").size(14).color(Color::BLACK)
                     ].spacing(10).align_y(iced::Alignment::Center)
                 )
                 .width(Length::Fill)
                 .padding(15)
                 .style(theme::button_primary)
                 .on_press(Message::ToggleTimer)
             );
        } else {
             col = col.push(
                 row![
                     button(text(if state.timer.is_running { "||" } else { ">" })).on_press(Message::ToggleTimer).style(theme::button_secondary),
                 ].spacing(10).padding(5)
             );
        }

        col.into()
    }

    fn view_tasks<'a>(&self, state: &'a State) -> Element<'a, Message> {
        let header = row![
            text("PRIORITY TASKS").size(12).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT }).color(theme::TEXT_DIM),
            horizontal_space(),
            text(format!("{} Remaining", state.tasks.len())).size(10).color(theme::TEXT_DIM)
        ].align_y(iced::Alignment::Center).width(Length::Fill);

        let items: Element<'a, Message> = if state.tasks.is_empty() {
             text("No active tasks.").size(14).color(theme::TEXT_DIM).into()
        } else {
             scrollable(column(
                 state.tasks.iter().map(|task| {
                     let is_active = state.active_task_id == Some(task.id);
                     row![
                         // Active Toggle
                         button(
                             container(horizontal_space().width(8).height(8))
                                .style(move |_t: &Theme| container::Style { background: Some(if is_active { state.primary_color } else { Color::TRANSPARENT }.into()), ..container::Style::default() })
                         )
                         .style(theme::button_secondary)
                         .width(24).height(24)
                         .on_press(Message::SetActiveTask(task.id)),

                         column![
                             text(&task.text).size(14).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT }),
                             text(if is_active { "Active Task" } else { "Focus on this task" }).size(10).color(theme::TEXT_DIM)
                         ].spacing(2),

                         horizontal_space(),

                         button(text("Done").size(10)).on_press(Message::MarkTaskDone(task.id)).style(theme::button_secondary),
                         button(text("Scrap").size(10)).on_press(Message::DeleteTask(task.id)).style(theme::button_ghost)
                     ]
                     .spacing(15)
                     .align_y(iced::Alignment::Center)
                     .padding(10)
                     .into()
                 })
             ).spacing(10)).height(Length::Fill).into()
        };

        let input = row![
            text_input("Add a new task...", &state.new_task_input)
                .on_input(Message::UpdateNewTaskInput)
                .on_submit(Message::AddTask)
                .padding(10),
            button(text("+")).on_press(Message::AddTask).style(theme::button_secondary)
        ].spacing(10);

        column![
            header,
            container(horizontal_space().height(1)).style(|_t: &Theme| container::Style { background: Some(theme::TEXT_DIM.into()), ..container::Style::default() }).width(Length::Fill),
            items,
            horizontal_space().height(10),
            input
        ].spacing(15).into()
    }

    fn view_footer<'a>(&self, state: &'a State) -> Element<'a, Message> {
        let hours = state.session_focus_seconds / 3600;
        let mins = (state.session_focus_seconds % 3600) / 60;

        let stats = column![
            text("CURRENT SESSION").size(10).color(theme::TEXT_DIM).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT }),
            text(format!("{:02}:{:02} Total Focus Time Today", hours, mins)).size(12)
        ].spacing(2);

        let settings_row = if state.settings_open {
             row![
                 button(container(horizontal_space().width(10).height(10)).style(|_: &Theme| container::Style{ background: Some(theme::ORANGE.into()), border: iced::Border{radius: 10.0.into(), ..iced::Border::default()}, ..container::Style::default() }))
                    .on_press(Message::SetColor(theme::ORANGE)).style(theme::button_ghost),
                 button(container(horizontal_space().width(10).height(10)).style(|_: &Theme| container::Style{ background: Some(theme::CYAN.into()), border: iced::Border{radius: 10.0.into(), ..iced::Border::default()}, ..container::Style::default() }))
                    .on_press(Message::SetColor(theme::CYAN)).style(theme::button_ghost),
                 button(container(horizontal_space().width(10).height(10)).style(|_: &Theme| container::Style{ background: Some(Color::from_rgb(0.5, 0.0, 1.0).into()), border: iced::Border{radius: 10.0.into(), ..iced::Border::default()}, ..container::Style::default() }))
                    .on_press(Message::SetColor(Color::from_rgb(0.5, 0.0, 1.0))).style(theme::button_ghost),
             ].spacing(5)
        } else {
            row![].into()
        };

        row![
            stats,
            horizontal_space(),
            settings_row,
            row![
                button(text("Contrast")).on_press(Message::ToggleTheme).style(theme::button_secondary).width(60).height(40),
                button(text("Settings")).on_press(Message::ToggleSettings).style(theme::button_secondary).width(60).height(40),
            ].spacing(8)
        ]
        .align_y(iced::Alignment::End)
        .width(Length::Fill)
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self {
            PomimiApp::Loaded(state) if state.timer.is_running => {
                time::every(Duration::from_secs(1)).map(|_| Message::Tick)
            }
            _ => Subscription::none(),
        }
    }

    pub fn theme(&self) -> Theme {
        match self {
            PomimiApp::Loaded(state) => {
                theme::create_theme(state.is_dark_mode, state.primary_color)
            },
            _ => theme::create_theme(true, theme::ORANGE),
        }
    }
}
