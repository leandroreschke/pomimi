# POMIMI 🍅

A simple, native, and aesthetic Pomodoro timer written in Rust.

## Features
- **Zero Dependencies**: Built with standard Rust libraries only.
- **Beautiful UI**: Gradient progress bars (Indigo to Sunset Orange) with dithered backgrounds.
- **Interactive CLI**: Menu navigation using Vim keys (`j`/`k`) or Arrows.
- **Custom Timers**: Support for quick custom durations via arguments.

## How to Run

### Prerequisites
You need to have Rust installed. If you don't, install it from [rustup.rs](https://rustup.rs).

### Interactive Mode
Run the app with the interactive menu:
```bash
cargo run
```

### Custom Timer
Run a specific timer duration directly:
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
pomimi
pomimi 45m
```

## How to Share

### Option 1: Share the Source Code (Recommended for Developers)
If your friends have Rust installed, simply share this folder (or upload it to GitHub). They can run it using:
```bash
cargo run
```

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
4.  Send this file to your friends! They can run it directly from their terminal.
