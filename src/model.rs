use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: u64,
    pub text: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    pub name: String,
    pub items: Vec<TodoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub cli_mode_default: bool,
    pub gui_always_on_top: bool,
    pub run_in_mini_mode: bool,
    pub todo_lists: Vec<TodoList>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cli_mode_default: false,
            gui_always_on_top: false,
            run_in_mini_mode: false,
            todo_lists: vec![TodoList {
                name: "Main".to_string(),
                items: Vec::new(),
            }],
        }
    }
}

impl Config {
    fn get_config_path() -> Option<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "pomimi", "pomimi") {
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() {
                let _ = fs::create_dir_all(config_dir);
            }
            Some(config_dir.join("config.json"))
        } else {
            None
        }
    }

    pub fn load() -> Self {
        if let Some(path) = Self::get_config_path() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::get_config_path() {
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, content);
            }
        }
    }
}
