# POMIMI 🍅

A simple, native, and aesthetic Pomodoro timer and Task manager written in Rust.

## Features
- **Native GUI**: Built with [Iced](https://github.com/iced-rs/iced), featuring a clean, minimal interface.
- **Task Management**: Keep track of your priority tasks directly within the timer.
- **Focus Timer**: Standard Pomodoro intervals (25/5) or long sessions (50/10).
- **Mini Mode**: A compact, always-on-top window to keep an eye on time without distractions.
- **Session Stats**: Track your daily focus time.
- **CLI Support**: Prefer the terminal? The original CLI mode is still fully supported.

## How to Run

### Prerequisites
You need to have Rust installed. If you don't, install it from [rustup.rs](https://rustup.rs).

### GUI Mode (Default)
Run the application with the graphical interface:
```bash
cargo run
```

### CLI Mode
Run the app in the terminal:
```bash
cargo run -- --cli
```
Or run a specific timer duration directly in CLI mode:
```bash
cargo run -- 15m   # Run for 15 minutes
cargo run -- 30s   # Run for 30 seconds
```

## How to Install

To install `pomimi` globally on your system so you can run it from anywhere just by typing `pomimi`:

1.  Navigate to the project directory.
2.  Run the install command:
    ```bash
    cargo install --path .
    ```

Now you can use it anywhere:
```bash
pomimi           # Launches GUI
pomimi --cli     # Launches Interactive CLI
pomimi 45m       # Launches CLI Timer for 45 mins
```

## How to Share

### Option 1: Share the Source Code (Recommended for Developers)
If your friends have Rust installed, simply share this folder (or upload it to GitHub). They can run it using `cargo run`.

### Option 2: Share the Binary (For Non-Developers)
You can build a standalone executable that works without Rust installed.

1.  Build the release version:
    ```bash
    cargo build --release
    ```
2.  Navigate to the build folder:
    ```bash
    cd target/release
    ```
3.  You will find a file named `pomimi` (or `pomimi.exe` on Windows).
4.  Send this file to your friends! They can run it directly.
