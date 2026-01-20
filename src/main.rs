use std::env;
use std::io::{self, Read, Write};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

// --- Terminal Handling ---

struct RawMode;

impl RawMode {
    fn enable() -> io::Result<Self> {
        let status = Command::new("stty")
            .arg("-icanon")
            .arg("-echo")
            .status()?;

        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to set raw mode"));
        }
        Ok(RawMode)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = Command::new("stty").arg("sane").status();
    }
}

// --- Input Handling ---

enum Key {
    Up,
    Down,
    Char(char),
    Enter,
    Unknown,
}

fn read_key() -> Key {
    let mut buffer = [0; 1];
    let mut stdin = io::stdin();
    
    if stdin.read(&mut buffer).is_ok() {
        match buffer[0] {
            b'\n' | b'\r' => Key::Enter,
            b'\x1b' => {
                let mut seq = [0; 2];
                if stdin.read(&mut seq).is_ok() {
                    if seq[0] == b'[' {
                        match seq[1] {
                            b'A' => Key::Up,
                            b'B' => Key::Down,
                            _ => Key::Unknown,
                        }
                    } else {
                        Key::Unknown
                    }
                } else {
                    Key::Char('\x1b')
                }
            }
            c => Key::Char(c as char),
        }
    } else {
        Key::Unknown
    }
}

// --- Time Helpers ---

fn get_current_time_str() -> String {
    let output = Command::new("date")
        .arg("+%H:%M")
        .output()
        .expect("Failed to execute date command");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn get_future_time_str(add_seconds: u64) -> String {
    let arg = format!("-v+{}S", add_seconds);
    let output = Command::new("date")
        .arg(&arg)
        .arg("+%H:%M")
        .output()
        .expect("Failed to execute date command");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// --- Graphics & Logic ---

#[derive(Clone, Copy)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    fn lerp(&self, other: &Rgb, t: f32) -> Rgb {
        Rgb {
            r: (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
            g: (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
            b: (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
        }
    }
}

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().unwrap();
}

fn hide_cursor() {
    print!("\x1b[?25l");
    io::stdout().flush().unwrap();
}

fn show_cursor() {
    print!("\x1b[?25h");
    io::stdout().flush().unwrap();
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn draw_progress_bar(width: usize, progress: f32, start_color: Rgb, end_color: Rgb) {
    let filled_width = (width as f32 * progress).round() as usize;
    
    for i in 0..width {
        if i < filled_width {
            let t = i as f32 / width as f32;
            let color = start_color.lerp(&end_color, t);
            // Solid block for progress
            print!("\x1b[38;2;{};{};{}m\u{2588}", color.r, color.g, color.b);
        } else {
            // Dithered background, white/default color
            // Using light shade
            print!("\x1b[37m\u{2591}"); 
        }
    }
    
    // Print percentage at the end
    let percent = (progress * 100.0) as usize;
    print!("\x1b[0m\x1b[2m{:>3}%\x1b[0m", percent); 
    
    io::stdout().flush().unwrap();
}

fn play_sound() {
    let _ = Command::new("afplay")
        .arg("/System/Library/Sounds/Glass.aiff")
        .spawn(); 
}

fn wait_for_user_approval() {
    println!("\n\n\x1b[2mReady to start? [Press Enter]\x1b[0m");
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

fn run_timer(duration: Duration, label: &str, require_approval: bool) {
    if require_approval {
        wait_for_user_approval();
    }

    let start_instant = Instant::now();
    let indigo = Rgb { r: 75, g: 0, b: 130 };
    let orange = Rgb { r: 253, g: 94, b: 83 };
    let width = 25;

    let start_time_str = get_current_time_str();
    let predicted_end_str = get_future_time_str(duration.as_secs());

    hide_cursor();

    loop {
        let elapsed = start_instant.elapsed();
        if elapsed >= duration {
            break;
        }

        let remaining = duration - elapsed;
        let progress = elapsed.as_secs_f32() / duration.as_secs_f32();

        clear_screen();
        // Top Left: Start Time (No label)
        println!("\x1b[2m{}\x1b[0m\n", start_time_str);

        println!("\x1b[2mPOMIMI: {}\x1b[0m", label);
        
        // Time Remaining with Predicted End on Left (No label for End)
        println!("\x1b[2m{}  Time Remaining: {}\x1b[0m\n", predicted_end_str, format_duration(remaining));
        
        draw_progress_bar(width, progress, indigo, orange);
        
        thread::sleep(Duration::from_millis(100));
    }
    
    // Final state
    clear_screen();
    println!("\x1b[2m{}\x1b[0m\n", start_time_str);
    println!("\x1b[2mPOMIMI: {} - DONE!\x1b[0m", label);
    println!("\x1b[2m{}  Time Remaining: 00:00\x1b[0m\n", predicted_end_str);
    draw_progress_bar(width, 1.0, indigo, orange);
    println!("\n");
    
    play_sound();
    
    show_cursor();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check custom arg
    if args.len() > 1 {
        let time_str = &args[1];
        let duration = if time_str.ends_with('m') {
            let mins = time_str.trim_end_matches('m').parse::<u64>().unwrap_or(0);
            Duration::from_secs(mins * 60)
        } else if time_str.ends_with('s') {
            let secs = time_str.trim_end_matches('s').parse::<u64>().unwrap_or(0);
            Duration::from_secs(secs)
        } else {
             let mins = time_str.parse::<u64>().unwrap_or(0);
             Duration::from_secs(mins * 60)
        };

        if duration.as_secs() > 0 {
             run_timer(duration, "Custom Focus", false);
             return;
        } else {
            eprintln!("Invalid time format. Use '15m' or '30s'.");
            return;
        }
    }

    // Menu State
    let mut require_approval = false;
    let mut selection = 0;
    
    let _raw = match RawMode::enable() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Could not enable raw mode: {}", e);
            return;
        }
    };

    hide_cursor();
    
    loop {
        clear_screen();
        println!("\x1b[2mChoose a POMIMI time:\x1b[0m\n");

        let items_display = [
            "25/5 Classic",
            "50/10 Long",
            if require_approval { "Require Input: ON" } else { "Require Input: OFF" },
            "Quit"
        ];

        for (i, label) in items_display.iter().enumerate() {
            if i == selection {
                println!("> {} \x1b[32m[Selected]\x1b[0m", label);
            } else {
                println!("  {}", label);
            }
        }
        
        println!("\n\x1b[2m(Use j/k/arrows to move, Enter/A to select)\x1b[0m");

        io::stdout().flush().unwrap();

        match read_key() {
            Key::Up | Key::Char('k') => {
                if selection > 0 {
                    selection -= 1;
                }
            }
            Key::Down | Key::Char('j') => {
                if selection < 3 {
                    selection += 1;
                }
            }
            Key::Enter | Key::Char('a') | Key::Char('A') => {
                match selection {
                    0 => { // Classic
                        drop(_raw); 
                        run_timer(Duration::from_secs(25 * 60), "Focus (25m)", false); 
                        run_timer(Duration::from_secs(5 * 60), "Break (5m)", require_approval);
                        break;
                    },
                    1 => { // Long
                        drop(_raw);
                        run_timer(Duration::from_secs(50 * 60), "Focus (50m)", false);
                        run_timer(Duration::from_secs(10 * 60), "Break (10m)", require_approval);
                        break;
                    },
                    2 => { // Toggle
                        require_approval = !require_approval;
                    },
                    3 => { // Quit
                        break;
                    },
                    _ => {}
                }
            }
            Key::Char('q') => {
                break;
            }
            _ => {}
        }
    }
    
    show_cursor();
}
