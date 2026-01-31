use iced::{Element, Task, Theme, Subscription, time, Length, window, Size, Color, Point, Event};
use iced::widget::{column, container, text, button, center, row, text_input, scrollable, Space, stack};
use crate::theme;
use crate::model::{Database, Task as DbTask};
use crate::components;
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
}

fn play_sound() {
    std::thread::spawn(|| {
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("afplay")
                .arg("/System/Library/Sounds/Glass.aiff")
                .status();
        }

        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("powershell")
                .args(&["-c", "(New-Object Media.SoundPlayer 'C:\\Windows\\Media\\notify.wav').PlaySync()"])
                .status();
        }

        #[cfg(target_os = "linux")]
        {
            let paths = [
                 "/usr/share/sounds/freedesktop/stereo/complete.oga",
                 "/usr/share/sounds/gnome/default/alerts/glass.ogg"
            ];
            let mut played = false;
            for path in paths {
                 if std::path::Path::new(path).exists() {
                     if let Ok(_) = std::process::Command::new("paplay").arg(path).status() {
                         played = true;
                         break;
                     }
                     if let Ok(_) = std::process::Command::new("aplay").arg(path).status() {
                         played = true;
                         break;
                     }
                 }
            }

            if !played {
                // Fallback to generated sound
                generate_and_play_beep();
            }
        }
    });
}

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
fn generate_and_play_beep() {
    let sample_rate = 44100;
    let duration_secs = 1.5; // Increased to 1.5s
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let mut data = Vec::with_capacity(44 + num_samples * 2);

    // RIFF
    data.extend_from_slice(b"RIFF");
    let file_size = 36 + num_samples * 2;
    data.extend_from_slice(&(file_size as u32).to_le_bytes());
    data.extend_from_slice(b"WAVE");

    // fmt
    data.extend_from_slice(b"fmt ");
    data.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    data.extend_from_slice(&1u16.to_le_bytes()); // PCM
    data.extend_from_slice(&1u16.to_le_bytes()); // Channels
    data.extend_from_slice(&(sample_rate as u32).to_le_bytes()); // Sample Rate
    let byte_rate = sample_rate * 2;
    data.extend_from_slice(&(byte_rate as u32).to_le_bytes());
    data.extend_from_slice(&2u16.to_le_bytes()); // Block align
    data.extend_from_slice(&16u16.to_le_bytes()); // Bits per sample

    // data
    data.extend_from_slice(b"data");
    let data_size = num_samples * 2;
    data.extend_from_slice(&(data_size as u32).to_le_bytes());

    // Sine wave 440Hz with simple envelope
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let mut amplitude = 0.2 * 32767.0;

        // Fade out
        if t > 1.0 {
            amplitude *= (1.5 - t) / 0.5;
        }

        let value = (amplitude * (2.0 * std::f32::consts::PI * 440.0 * t).sin()) as i16;
        data.extend_from_slice(&value.to_le_bytes());
    }

    let mut path = std::env::temp_dir();
    path.push("pomimi_beep.wav");

    if let Ok(_) = std::fs::write(&path, data) {
         let _ = std::process::Command::new("aplay").arg("-q").arg(&path).status();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    None,
    AddTask,
    Settings,
    TimerSettings,
}

#[derive(Debug, Clone)]
pub struct TimerState {
    pub phase: Phase,
    pub remaining_secs: u64,
    pub total_secs: u64,
    pub is_running: bool,
    pub cycles_completed: usize,
    pub waiting_for_user: bool,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            phase: Phase::Focus,
            remaining_secs: Phase::Focus.duration_secs(),
            total_secs: Phase::Focus.duration_secs(),
            is_running: false,
            cycles_completed: 0,
            waiting_for_user: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct State {
    pub db: Database,
    pub tasks: Vec<DbTask>,
    pub timer: TimerState,
    pub session_focus_seconds: i64,
    pub view_mode: ViewMode,
    pub new_task_input: String,
    pub active_task_id: Option<i64>,
    pub pending_completion_task_id: Option<i64>,
    pub active_modal: Modal,
    pub primary_color: Color,
    pub is_dark_mode: bool,
    pub require_confirmation: bool,
    pub window_position: Point,
    pub window_size: Size,
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
    ColorLoaded(Result<Option<(f32, f32, f32)>, String>),
    RequireConfirmationLoaded(Result<bool, String>),
    ThemeLoaded(Result<Option<bool>, String>),
    TaskOperationFailed(String),
    TaskOperationSuccess,

    // Timer
    ToggleTimer,
    Tick,
    SetDuration(u64),
    ResetTimer,

    // Tasks
    UpdateNewTaskInput(String),
    AddTask,
    SetActiveTask(i64),
    RequestCompleteTask(i64),
    ConfirmCompleteTask,
    CancelCompleteTask,

    // UI
    ToggleMiniMode,
    OpenModal(Modal),
    CloseModal,
    SetColor(Color),
    SetRequireConfirmation(bool),
    ToggleTheme,
    DragWindow,
    WindowMoved(Point),
    WindowResized(Size),

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
                        let load_color = Task::perform(
                             {
                                let db = db.clone();
                                async move { 
                                    db.get_accent_color().await.map_err(|e| e.to_string())
                                }
                             },
                             Message::ColorLoaded
                        );
                        let load_req_conf = Task::perform(
                             {
                                let db = db.clone();
                                async move {
                                    db.get_require_confirmation().await.map_err(|e| e.to_string())
                                }
                             },
                             Message::RequireConfirmationLoaded
                        );
                        let load_theme = Task::perform(
                             {
                                let db = db.clone();
                                async move {
                                    db.get_theme_preference().await.map_err(|e| e.to_string())
                                }
                             },
                             Message::ThemeLoaded
                        );

                        *self = PomimiApp::Loaded(State {
                            db,
                            tasks: Vec::new(),
                            timer: TimerState::default(),
                            session_focus_seconds: 0,
                            view_mode: ViewMode::Full,
                            new_task_input: String::new(),
                            active_task_id: None,
                            pending_completion_task_id: None,
                            active_modal: Modal::None,
                            primary_color: theme::ORANGE,
                            is_dark_mode: true,
                            require_confirmation: false,
                            window_position: Point::ORIGIN,
                            window_size: Size::new(380.0, 800.0),
                        });

                        Task::batch(vec![load_tasks, load_session, load_color, load_req_conf, load_theme])
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
                    Message::ColorLoaded(Ok(Some((r, g, b)))) => {
                        state.primary_color = Color::from_rgb(r, g, b);
                        Task::none()
                    }
                    Message::ColorLoaded(Ok(None)) => {
                        Task::none()
                    }
                    Message::ColorLoaded(Err(e)) => {
                        eprintln!("Failed to load color: {}", e);
                        Task::none()
                    }
                    Message::RequireConfirmationLoaded(Ok(val)) => {
                        state.require_confirmation = val;
                        Task::none()
                    }
                    Message::RequireConfirmationLoaded(Err(e)) => {
                        eprintln!("Failed to load require confirmation: {}", e);
                        Task::none()
                    }
                    Message::ThemeLoaded(Ok(Some(is_dark))) => {
                        state.is_dark_mode = is_dark;
                        Task::none()
                    }
                    Message::ThemeLoaded(Ok(None)) => Task::none(),
                    Message::ThemeLoaded(Err(e)) => {
                        eprintln!("Failed to load theme: {}", e);
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
                        if state.timer.waiting_for_user {
                            state.timer.waiting_for_user = false;
                            state.timer.is_running = true;
                        } else {
                            state.timer.is_running = !state.timer.is_running;
                        }
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
                                // Play sound
                                play_sound();

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
                                    }
                                    Phase::ShortBreak | Phase::LongBreak => {
                                        state.timer.phase = Phase::Focus;
                                    }
                                }
                                state.timer.remaining_secs = state.timer.phase.duration_secs();
                                state.timer.total_secs = state.timer.phase.duration_secs();

                                if state.require_confirmation {
                                    state.timer.is_running = false;
                                    state.timer.waiting_for_user = true;
                                }
                            }
                        }
                        Task::none()
                    }
                    Message::SetDuration(secs) => {
                        state.timer.is_running = false;
                        state.timer.phase = Phase::Focus;
                        state.timer.remaining_secs = secs;
                        state.timer.total_secs = secs;
                        state.active_modal = Modal::None;
                        Task::none()
                    }
                    Message::ResetTimer => {
                        state.timer.is_running = false;
                        state.timer.remaining_secs = state.timer.phase.duration_secs();
                        state.timer.total_secs = state.timer.phase.duration_secs();
                        state.timer.waiting_for_user = false;
                        state.active_modal = Modal::None;
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
                            state.active_modal = Modal::None; // Close modal
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
                    Message::SetActiveTask(id) => {
                        state.active_task_id = Some(id);
                        Task::none()
                    }
                    Message::RequestCompleteTask(id) => {
                        state.pending_completion_task_id = Some(id);
                        Task::none()
                    }
                    Message::CancelCompleteTask => {
                        state.pending_completion_task_id = None;
                        Task::none()
                    }
                    Message::ConfirmCompleteTask => {
                        if let Some(id) = state.pending_completion_task_id {
                            state.pending_completion_task_id = None;
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
                        } else {
                            Task::none()
                        }
                    }

                    // UI
                    Message::ToggleMiniMode => {
                        match state.view_mode {
                            ViewMode::Full => {
                                state.view_mode = ViewMode::Mini;
                                let mini_size = Size::new(270.0, 120.0);
                                let current_pos = state.window_position;
                                let current_size = state.window_size;
                                // Anchor Top-Right logic:
                                // new_x = old_x + old_width - new_width
                                // new_y = old_y (Top stays same)
                                let new_x = current_pos.x + current_size.width - mini_size.width;
                                let new_pos = Point::new(new_x, current_pos.y);

                                window::latest().and_then(move |id| {
                                    Task::batch(vec![
                                        window::resize(id, mini_size),
                                        window::set_level(id, window::Level::AlwaysOnTop),
                                        window::toggle_decorations(id),
                                        window::set_resizable(id, false),
                                        window::move_to(id, new_pos)
                                    ])
                                })
                            }
                            ViewMode::Mini => {
                                state.view_mode = ViewMode::Full;
                                let full_size = Size::new(380.0, 800.0);
                                let current_pos = state.window_position;
                                let current_size = state.window_size;
                                // Anchor Top-Right logic:
                                // new_x = old_x + old_width - new_width
                                // new_y = old_y
                                let new_x = current_pos.x + current_size.width - full_size.width;
                                let new_pos = Point::new(new_x, current_pos.y);

                                window::latest().and_then(move |id| {
                                    Task::batch(vec![
                                        window::resize(id, full_size),
                                        window::set_level(id, window::Level::Normal),
                                        window::toggle_decorations(id),
                                        window::set_resizable(id, true),
                                        window::move_to(id, new_pos)
                                    ])
                                })
                            }
                        }
                    }
                    Message::OpenModal(modal) => {
                        state.active_modal = modal;
                        Task::none()
                    }
                    Message::CloseModal => {
                        state.active_modal = Modal::None;
                        Task::none()
                    }
                    Message::SetColor(color) => {
                        state.primary_color = color;
                        let db = state.db.clone();
                        Task::perform(
                            async move {
                                db.save_accent_color(color.r, color.g, color.b).await.map_err(|e| e.to_string())
                            },
                            |result| {
                                if let Err(e) = result {
                                    eprintln!("Failed to save color: {}", e);
                                }
                                Message::None
                            }
                        )
                    }
                    Message::SetRequireConfirmation(val) => {
                        state.require_confirmation = val;
                        let db = state.db.clone();
                        Task::perform(
                            async move {
                                db.save_require_confirmation(val).await.map_err(|e| e.to_string())
                            },
                            |result| {
                                if let Err(e) = result {
                                    eprintln!("Failed to save preference: {}", e);
                                }
                                Message::None
                            }
                        )
                    }
                    Message::ToggleTheme => {
                        state.is_dark_mode = !state.is_dark_mode;
                        let db = state.db.clone();
                        let is_dark = state.is_dark_mode;
                        Task::perform(
                            async move {
                                db.save_theme(is_dark).await.map_err(|e| e.to_string())
                            },
                            |result| {
                                if let Err(e) = result {
                                    eprintln!("Failed to save theme: {}", e);
                                }
                                Message::None
                            }
                        )
                    }
                    Message::DragWindow => {
                        window::latest().and_then(window::drag)
                    }
                    Message::WindowMoved(point) => {
                        state.window_position = point;
                        Task::none()
                    }
                    Message::WindowResized(size) => {
                        state.window_size = size;
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
            PomimiApp::Error(e) => center(text(format!("Error: {}", e)).size(20).color(theme::DANGER_RED)).into(),
            PomimiApp::Loaded(state) => {
                let timer_view = self.view_timer(state);

                // Background Text - dynamic based on phase
                let phase_bg_text = match state.timer.phase {
                    Phase::Focus => "FOCUS",
                    Phase::ShortBreak | Phase::LongBreak => "REST",
                };
                let background_text = container(
                    text(phase_bg_text)
                        .size(150)
                        .font(iced::Font { family: iced::font::Family::Name("Space Grotesk"), weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT })
                        .color(Color { a: 0.05, ..theme::WHITE })
                ).align_x(iced::Alignment::Center);

                let content: Element<Message> = if state.view_mode == ViewMode::Mini {
                    let active_task_view: Element<'_, Message> = match state.timer.phase {
                        Phase::ShortBreak | Phase::LongBreak => {
                             container(
                                 text("REST").size(12).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT })
                             )
                             .padding(10)
                             .width(Length::Fill)
                             .align_x(iced::Alignment::Center)
                             .style(|_t: &Theme| container::Style { background: Some(Color{a:0.05, ..theme::WHITE}.into()), ..container::Style::default() })
                             .into()
                        }
                        Phase::Focus => {
                            if let Some(id) = state.active_task_id {
                                if let Some(task) = state.tasks.iter().find(|t| t.id == id) {
                                     container(
                                         row![
                                             container(Space::new().width(6).height(6))
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
                                    Space::new().width(Length::Fill).into()
                                }
                            } else {
                                Space::new().width(Length::Fill).into()
                            }
                        }
                    };

                    iced::widget::mouse_area(
                        container(
                            column![
                                // Timer + play/pause + exit button in row
                                row![
                                    timer_view,
                                    button(text(
                                        if state.timer.waiting_for_user { "\u{e5c8}" }
                                        else if state.timer.is_running { "\u{e034}" }
                                        else { "\u{e037}" }
                                    ).font(iced::Font::with_name("Material Symbols Outlined")))
                                        .on_press(Message::ToggleTimer).style(components::button::secondary),
                                    button(text("\u{e895}").font(iced::Font::with_name("Material Symbols Outlined")).size(14))
                                        .on_press(Message::ToggleMiniMode)
                                        .style(components::button::tertiary)
                                ].width(Length::Fill).align_y(iced::Alignment::Center),
                                active_task_view,
                            ]
                            .align_x(iced::Alignment::Center)
                            .spacing(10)
                            .padding(10)
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(|t: &Theme| {
                            let base = theme::container_default(t);
                            container::Style {
                                border: iced::Border {
                                    radius: 20.0.into(),
                                    width: 1.0,
                                    color: t.palette().text,
                                },
                                background: Some(t.palette().background.into()),
                                ..base
                            }
                        })
                    )
                    .on_press(Message::DragWindow)
                    .into()

                } else {
                    let tasks_view = self.view_tasks(state);
                    let footer = self.view_footer(state);

                    let main_content = column![
                        timer_view,
                        Space::new().height(20),
                        tasks_view,
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
                };

                // Modal Overlay
                if state.active_modal != Modal::None {
                    let modal_content = match state.active_modal {
                        Modal::AddTask => {
                            column![
                                text("Add New Task").size(18).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT }),
                                text_input("What needs focus?", &state.new_task_input)
                                    .on_input(Message::UpdateNewTaskInput)
                                    .on_submit(Message::AddTask)
                                    .padding(10),
                                row![
                                    button(text("Cancel")).on_press(Message::CloseModal).style(components::button::secondary),
                                    button(text("Add Task")).on_press(Message::AddTask).style(components::button::primary)
                                ].spacing(10).align_y(iced::Alignment::Center)
                            ].spacing(20)
                        },
                        Modal::Settings => {
                            let color_btn = |color: Color| {
                                button(container(Space::new().width(20).height(20)).style(move |_: &Theme| container::Style{ background: Some(color.into()), border: iced::Border{radius: 20.0.into(), ..iced::Border::default()}, ..container::Style::default() }))
                                    .on_press(Message::SetColor(color)).style(components::button::tertiary)
                            };

                            column![
                                text("Settings").size(18).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT }),
                                text("Accent Color").size(14),
                                row![
                                     color_btn(theme::ORANGE),
                                     color_btn(theme::GREEN),
                                     color_btn(theme::BLUE),
                                     color_btn(theme::PINK),
                                     color_btn(theme::YELLOW),
                                     color_btn(theme::CYAN),
                                     color_btn(theme::PURPLE),
                                 ].spacing(10).wrap(),
                                 text("Theme").size(14),
                                 button(
                                     row![
                                         text(if state.is_dark_mode { "\u{e518}" } else { "\u{e51c}" }).font(iced::Font::with_name("Material Symbols Outlined")).size(18),
                                         text(if state.is_dark_mode { "Dark Mode" } else { "Light Mode" }).size(14)
                                     ].spacing(10).align_y(iced::Alignment::Center)
                                 )
                                 .on_press(Message::ToggleTheme)
                                 .style(components::button::secondary)
                                 .width(Length::Fill)
                                 .padding(10),

                                 row![
                                     components::checkbox::checkbox(state.require_confirmation, state.primary_color, Message::SetRequireConfirmation(!state.require_confirmation)),
                                     text("Require confirmation before phase change").size(14)
                                 ].spacing(10).align_y(iced::Alignment::Center),

                                 button(text("Done")).on_press(Message::CloseModal).style(components::button::primary).width(Length::Fill)
                            ].spacing(20)
                        },
                        Modal::TimerSettings => {
                             column![
                                text("Timer Settings").size(18).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT }),
                                text("Strategy").size(14),
                                row![
                                    button(text("25/5")).on_press(Message::SetDuration(25*60)).style(components::button::secondary).width(Length::Fill),
                                    button(text("50/10")).on_press(Message::SetDuration(50*60)).style(components::button::secondary).width(Length::Fill),
                                ].spacing(10),
                                Space::new().height(10),
                                button(text("Reset Timer")).on_press(Message::ResetTimer).style(components::button::secondary).width(Length::Fill),
                                Space::new().height(10),
                                button(text("Close")).on_press(Message::CloseModal).style(components::button::primary).width(Length::Fill)
                            ].spacing(20)
                        },
                        Modal::None => column![].into(),
                    };

                    let overlay = container(
                        container(modal_content)
                            .padding(20)
                            .style(theme::container_default) // Needs border to separate from bg? Using default for now
                            .style(|t: &Theme| {
                                let base = theme::container_default(t);
                                container::Style {
                                    border: iced::Border { width: 1.0, color: t.palette().text, radius: 0.0.into() },
                                    ..base
                                }
                            })
                            .width(300)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center)
                    .style(|_t: &Theme| container::Style { background: Some(Color { a: 0.8, ..theme::BLACK }.into()), ..container::Style::default() });

                    stack![
                        content,
                        overlay
                    ].into()
                } else {
                    content.into()
                }
            }
        }
    }

    fn view_timer<'a>(&self, state: &'a State) -> Element<'a, Message> {
        components::timer::timer_display(state)
    }

    fn view_tasks<'a>(&self, state: &'a State) -> Element<'a, Message> {
        let header = row![
            text("PRIORITY TASKS").size(12).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT }).color(theme::TEXT_DIM),
            Space::new().width(Length::Fill),
            button(text("+").size(14)).on_press(Message::OpenModal(Modal::AddTask)).style(components::button::tertiary)
        ].align_y(iced::Alignment::Center).width(Length::Fill);

        let items: Element<'a, Message> = if state.tasks.is_empty() {
             container(text("No active tasks.").size(14).color(theme::TEXT_DIM)).width(Length::Fill).align_x(iced::Alignment::Center).padding(20).into()
        } else {
             scrollable(column(
                 state.tasks.iter().map(|task| {
                     if state.pending_completion_task_id == Some(task.id) {
                         row![
                             button(text("Complete").size(14))
                                 .on_press(Message::ConfirmCompleteTask)
                                 .style(components::button::primary)
                                 .width(Length::Fill)
                                 .padding(10),
                             button(text("\u{e5c9}").font(iced::Font::with_name("Material Symbols Outlined")).size(14))
                                 .on_press(Message::CancelCompleteTask)
                                 .style(components::button::secondary)
                                 .padding(10)
                         ]
                         .spacing(10)
                         .padding(10)
                         .width(Length::Fill)
                         .into()
                     } else {
                         let is_active = state.active_task_id == Some(task.id);
                         row![
                             components::checkbox::checkbox(is_active, state.primary_color, Message::SetActiveTask(task.id)),

                             container(
                                 column![
                                     text(&task.text)
                                         .size(14)
                                         .font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT })
                                         .style(move |t: &Theme| text::Style {
                                             color: if is_active { Some(t.palette().primary) } else { Some(theme::TEXT_DIM) },
                                             ..text::Style::default()
                                         })
                                         .wrapping(text::Wrapping::None),
                                 ].spacing(2)
                             )
                             .width(Length::Fill)
                             .clip(true),

                             button(text("\u{e876}").font(iced::Font::with_name("Material Symbols Outlined")).size(14))
                                .on_press(Message::RequestCompleteTask(task.id))
                                .style(components::button::tertiary)
                                .padding(5)
                         ]
                         .spacing(15)
                         .align_y(iced::Alignment::Center)
                         .padding(10)
                         .width(Length::Fill)
                         .into()
                     }
                 })
             ).spacing(10)).height(Length::Fill).into()
        };

        column![
            header,
            container(Space::new().height(1)).style(|_t: &Theme| container::Style { background: Some(theme::TEXT_DIM.into()), ..container::Style::default() }).width(Length::Fill),
            items,
        ].spacing(15).into()
    }

    fn view_footer<'a>(&self, state: &'a State) -> Element<'a, Message> {
        let hours = state.session_focus_seconds / 3600;
        let mins = (state.session_focus_seconds % 3600) / 60;

        let stats = column![
            text("CURRENT SESSION").size(10).color(theme::TEXT_DIM).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::DEFAULT }),
            text(format!("{:02}:{:02} Total Focus Time Today", hours, mins)).size(12)
        ].spacing(2);

        row![
            stats,
            Space::new().width(Length::Fill),
            row![
                // Settings Icon
                button(center(text("\u{e8b8}").font(iced::Font::with_name("Material Symbols Outlined")).size(18)))
                    .on_press(Message::OpenModal(Modal::Settings))
                    .style(components::button::secondary)
                    .padding(0)
                    .width(40).height(40),
                // Mini Mode Icon
                button(center(text("\u{e895}").font(iced::Font::with_name("Material Symbols Outlined")).size(18)))
                    .on_press(Message::ToggleMiniMode)
                    .style(components::button::secondary)
                    .padding(0)
                    .width(40).height(40),
            ].spacing(8)
        ]
        .align_y(iced::Alignment::End)
        .width(Length::Fill)
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let window_events = iced::event::listen().map(|event| match event {
            Event::Window(window::Event::Moved(point)) => Message::WindowMoved(point),
            Event::Window(window::Event::Resized(size)) => Message::WindowResized(size),
            _ => Message::None,
        });

        match self {
            PomimiApp::Loaded(state) => {
                let mut subs = vec![window_events];
                if state.timer.is_running {
                    subs.push(time::every(Duration::from_secs(1)).map(|_| Message::Tick));
                }
                Subscription::batch(subs)
            }
            _ => window_events,
        }
    }

    pub fn theme(&self) -> Theme {
        match self {
            PomimiApp::Loaded(state) => {
                let transparent_bg = state.view_mode == ViewMode::Mini;
                theme::create_theme(state.is_dark_mode, state.primary_color, transparent_bg)
            },
            _ => theme::create_theme(true, theme::ORANGE, false),
        }
    }
}
