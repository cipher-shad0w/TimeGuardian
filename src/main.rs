/*
* TimeGuardian - A website blocker for focused productivity
* Author: Jannis Krija (https://github.com/cipher-shad0w)
* 
* This application helps users stay focused by temporarily blocking distracting websites.
* It modifies the hosts file to redirect specified websites to localhost during focus sessions.
*/

mod tui;

use clap::{Parser, Subcommand};
use color_eyre::{eyre::Context, Result};
use crossterm::{
    event::{Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use directories::BaseDirs;
use ratatui::Terminal;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, stdout, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use tui_input::{backend::crossterm::EventHandler, Input};

// Local imports for our TUI module
use crate::tui::{App, TuiMode};

// Constants for file paths and configurations
const APP_NAME: &str = "timeguardian";
const HOSTS_BACKUP: &str = "hosts.backup";
const TEMP_HOSTS_MARKER: &str = "# ===== TimeGuardian Temporary Hosts =====";

/// TimeGuardian: A modern, user-friendly CLI application to block distracting websites 
/// and improve productivity by creating focused work sessions.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// The command to execute
    #[command(subcommand)]
    command: Option<Commands>,

    /// Blocking duration with units (e.g., 25m, 30s, 1h)
    #[arg(long = "duration", short = 'd')]
    duration: Option<String>,

    /// Task name or reason for the focus session
    #[arg(long = "task", short = 't')]
    task: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the TUI (text user interface)
    Tui,
    
    /// Set up the application with a website list
    Setup {
        /// Path to the file containing websites to block
        #[arg(long = "list")]
        list_path: String,
    },
    
    /// Reset hosts file to its original state
    Reset,
    
    /// Request sudo access and set up permissions
    #[command(alias = "perms")]
    Permissions,
}

/// Application configuration structure
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Config {
    website_list_path: String,
    website_lists: Option<Vec<tui::WebsiteList>>,
    use_sudo: Option<bool>,
}

/// Get the path to the hosts file based on the operating system
fn get_hosts_path() -> PathBuf {
    if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        PathBuf::from("/etc/hosts")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    } else {
        panic!("Unsupported operating system")
    }
}

/// Find or create the application's configuration directory
fn get_config_dir() -> Result<PathBuf> {
    if let Some(base_dirs) = BaseDirs::new() {
        let config_dir = base_dirs.config_dir().join(APP_NAME);
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .wrap_err_with(|| format!("Could not create configuration directory: {:?}", config_dir))?;
        }
        Ok(config_dir)
    } else {
        let fallback = env::current_dir()?.join(".config").join(APP_NAME);
        fs::create_dir_all(&fallback)?;
        Ok(fallback)
    }
}

/// Load configuration or return default configuration
fn load_config() -> Result<Config> {
    let config_path = get_config_dir()?.join("config.toml");
    
    if config_path.exists() {
        let config_content = fs::read_to_string(&config_path)
            .wrap_err_with(|| format!("Could not read configuration file: {:?}", config_path))?;
        
        let config: Config = toml::from_str(&config_content)
            .wrap_err("Could not parse configuration")?;
        
        Ok(config)
    } else {
        // Return default configuration
        Ok(Config {
            website_list_path: "websites.txt".to_string(),
            website_lists: None,
            use_sudo: Some(false),
        })
    }
}

/// Save configuration to file
fn save_config(config: &Config) -> Result<()> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");
    
    let toml_string = toml::to_string(config)
        .wrap_err("Could not serialize configuration")?;
    
    fs::write(&config_path, toml_string)
        .wrap_err_with(|| format!("Could not save configuration: {:?}", config_path))?;
    
    Ok(())
}

/// Initialize the website blocker application
fn initialize_app() -> Result<()> {
    let config_dir = get_config_dir()?;
    
    // Create backup file if it doesn't exist
    let backup_path = config_dir.join(HOSTS_BACKUP);
    if !backup_path.exists() {
        let hosts_content = fs::read_to_string(get_hosts_path())
            .wrap_err("Could not read hosts file")?;
        
        fs::write(&backup_path, hosts_content)
            .wrap_err_with(|| format!("Could not create hosts file backup: {:?}", backup_path))?;
    }
    
    Ok(())
}

/// Check if root permissions are required and request them if needed
fn check_and_get_permissions() -> Result<bool> {
    if cfg!(unix) {
        // Test if we can write to the hosts file
        match OpenOptions::new()
            .write(true)
            .open(get_hosts_path())
        {
            Ok(_) => Ok(true),
            Err(_) => {
                println!("This application needs write permissions for the hosts file.");
                println!("Do you want to run the application with sudo permissions? (y/n)");
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
                    // Find the current executable
                    let current_exe = env::current_exe()?;
                    
                    // Restart with sudo
                    let status = Command::new("sudo")
                        .arg(current_exe)
                        .args(env::args().skip(1))
                        .status()?;
                    
                    if status.success() {
                        std::process::exit(0);
                    } else {
                        println!("Running with sudo failed.");
                        Ok(false)
                    }
                } else {
                    println!("Without sufficient permissions, website blocking will not work.");
                    Ok(false)
                }
            }
        }
    } else {
        // On Windows and other systems, perform other permission checks
        Ok(true)
    }
}

/// Run blocker with timer
fn block_websites_with_timer(
    websites: &[String], 
    duration: Duration, 
    task_name: &str,
    duration_text: &str,
) -> Result<()> {
    // Check and get permissions if needed
    if !check_and_get_permissions()? {
        return Ok(());
    }

    let hosts_path = get_hosts_path();
    let config_dir = get_config_dir()?;
    let backup_path = config_dir.join(HOSTS_BACKUP);

    // Read current content of hosts file
    let mut hosts_content = fs::read_to_string(&hosts_path)
        .wrap_err_with(|| format!("Could not read hosts file: {:?}", hosts_path))?;

    // Create backup if it doesn't exist or is empty
    if !backup_path.exists() || fs::read_to_string(&backup_path)?.trim().is_empty() {
        fs::write(&backup_path, &hosts_content)
            .wrap_err_with(|| format!("Could not create backup: {:?}", backup_path))?;
    }

    // Remove previous temporary entries if present
    if let Some(start) = hosts_content.find(TEMP_HOSTS_MARKER) {
        if let Some(end) = hosts_content[start..].find("\n# ===== End") {
            hosts_content = hosts_content[..start].to_string() + &hosts_content[start + end + 12..];
        }
    }

    // Add new temporary entries
    hosts_content.push_str(&format!("\n{}\n", TEMP_HOSTS_MARKER));
    for website in websites {
        if !website.trim().is_empty() && !hosts_content.contains(website) {
            hosts_content.push_str(&format!("127.0.0.1\t{}\n", website));
        }
    }
    hosts_content.push_str(&format!("# ===== End Temporary Hosts =====\n"));

    // Write the updated hosts file
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&hosts_path)
        .wrap_err_with(|| format!("Could not open hosts file: {:?}", hosts_path))?;

    file.write_all(hosts_content.as_bytes())
        .wrap_err("Could not update hosts file")?;

    // Terminal output
    let message = format!(
        "Blocking websites for {} for task: {}",
        duration_text, task_name
    );
    
    let mut spinner = Spinner::new(Spinners::Dots12, message);
    
    // Start timer
    enable_raw_mode()?;
    let start_time = Instant::now();
    
    while start_time.elapsed() < duration {
        // Check for user input to end early
        if crossterm::event::poll(Duration::from_millis(100))? {
            let event = crossterm::event::read()?;
            if matches!(event, Event::Key(key) if key.code == KeyCode::Esc || key.code == KeyCode::Char('q')) {
                break;
            }
        }
        
        // Display remaining time (overwritten by spinner)
        let remaining = duration.checked_sub(start_time.elapsed()).unwrap_or_default();
        // The Spinner library doesn't support direct message changes
        // Create a new spinner with the updated message instead
        spinner.stop();
        spinner = Spinner::new(
            Spinners::Dots12,
            format!(
                "Remaining time: {:02}:{:02}:{:02}",
                remaining.as_secs() / 3600,
                (remaining.as_secs() % 3600) / 60,
                remaining.as_secs() % 60
            ),
        );
    }
    
    disable_raw_mode()?;
    spinner.stop();

    // Remove blocking after timer expires
    stop_blocking()?;
    
    println!("\nBlocking removed! ✅");
    
    Ok(())
}

/// Run the TUI application
fn run_tui() -> Result<()> {
    // Setup permissions first
    if !check_and_get_permissions()? {
        println!("The TUI cannot be started without the necessary permissions.");
        return Ok(());
    }
    
    // Initialize app data
    initialize_app()?;
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    
    // Create a terminal instance
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    
    // Initialize app
    app.init()?;
    
    // Load existing website lists from config if available
    let config = load_config()?;
    if let Some(website_lists) = config.website_lists {
        app.website_lists = website_lists;
        if !app.website_lists.is_empty() {
            app.website_list_state.select(Some(0));
            app.selected_list_index = Some(0);
            
            // Ensure the first list is properly selected
            if !app.website_lists[0].websites.is_empty() {
                app.website_state.select(Some(0));
                app.selected_website_index = Some(0);
            }
        }
    }
    
    // Create event handler
    let tick_rate = Duration::from_millis(250);
    let event_handler = tui::event::EventHandler::new(tick_rate);
    
    // Main loop
    while app.running {
        // Draw UI
        terminal.draw(|frame| tui::ui::render(&mut app, frame))?;
        
        // Handle events
        match event_handler.receiver.recv() {
            Ok(tui::event::Event::Key(key_event)) => {
                match app.mode {
                    TuiMode::Normal => match key_event.code {
                        KeyCode::Char('q') => {
                            app.running = false;
                        }
                        KeyCode::Char('?') => {
                            app.mode = TuiMode::Help;
                        }
                        KeyCode::Tab => {
                            app.tabs.next();
                        }
                        KeyCode::BackTab => {
                            app.tabs.previous();
                        }
                        _ => {
                            // Handle different tabs
                            match app.tabs.index {
                                0 => {
                                    handle_website_list_tab_events(&mut app, key_event.code)?;
                                }
                                1 => {
                                    handle_timer_tab_events(&mut app, key_event.code)?;
                                }
                                _ => {}
                            }
                        }
                    },
                    TuiMode::Editing => match key_event.code {
                        KeyCode::Esc => app.mode = TuiMode::Normal,
                        KeyCode::Enter => {
                            let input_value = app.input.value().to_string();
                            if !input_value.is_empty() {
                                match app.tabs.index {
                                    0 => {
                                        if app.selected_list_index.is_some() {
                                            app.add_website(input_value);
                                            app.status_message = "Website added successfully".to_string();
                                        } else {
                                            app.add_list(input_value);
                                            app.status_message = "List added successfully".to_string();
                                        }
                                    }
                                    _ => {}
                                }
                                app.input = Input::default();
                                app.mode = TuiMode::Normal;
                            }
                        }
                        // Handle other key events for input editing
                        _ => {
                            app.input.handle_event(&crossterm::event::Event::Key(key_event));
                        }
                    },
                    TuiMode::Help => match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                            app.mode = TuiMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
            Ok(tui::event::Event::Tick) => {
                app.tick();
                
                // Check if blocking session has ended
                if app.is_blocking {
                    if let Some(end_time) = app.blocking_end_time {
                        if Instant::now() >= end_time {
                            stop_blocking_websites()?;
                            app.stop_blocking()?;
                        }
                    }
                }
            }
            Ok(tui::event::Event::Resize(_, _)) => {}
            Err(_) => {
                app.running = false;
            }
        }
    }

    // When the app exits, save the website lists to config
    let mut config = load_config()?;
    config.website_lists = Some(app.website_lists.clone());
    save_config(&config)?;
    
    // Restore terminal
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    
    Ok(())
}

/// Handle key events for the website list tab
fn handle_website_list_tab_events(app: &mut App, key: KeyCode) -> Result<()> {
    match key {
        // Navigate lists
        KeyCode::Left => {
            app.website_state.select(None);
            app.selected_website_index = None;
        }
        KeyCode::Right => {
            if app.selected_list_index.is_some() {
                if let Some(list) = app.current_website_list() {
                    if !list.websites.is_empty() {
                        app.website_state.select(Some(0));
                        app.selected_website_index = Some(0);
                    }
                }
            }
        }
        KeyCode::Up => {
            if app.selected_website_index.is_some() {
                // Navigate websites
                let websites_len = app.current_website_list().map_or(0, |list| list.websites.len());
                if websites_len > 0 {
                    let i = app.selected_website_index.map_or(0, |i| {
                        if i > 0 { i - 1 } else { websites_len - 1 }
                    });
                    app.website_state.select(Some(i));
                    app.selected_website_index = Some(i);
                }
            } else {
                // Navigate lists
                let lists_len = app.website_lists.len();
                if lists_len > 0 {
                    let i = app.selected_list_index.map_or(0, |i| {
                        if i > 0 { i - 1 } else { lists_len - 1 }
                    });
                    app.website_list_state.select(Some(i));
                    app.selected_list_index = Some(i);
                }
            }
        }
        KeyCode::Down => {
            if app.selected_website_index.is_some() {
                // Navigate websites
                let websites_len = app.current_website_list().map_or(0, |list| list.websites.len());
                if websites_len > 0 {
                    let i = app.selected_website_index.map_or(0, |i| {
                        if i < websites_len - 1 { i + 1 } else { 0 }
                    });
                    app.website_state.select(Some(i));
                    app.selected_website_index = Some(i);
                }
            } else {
                // Navigate lists
                let lists_len = app.website_lists.len();
                if lists_len > 0 {
                    let i = app.selected_list_index.map_or(0, |i| {
                        if i < lists_len - 1 { i + 1 } else { 0 }
                    });
                    app.website_list_state.select(Some(i));
                    app.selected_list_index = Some(i);
                }
            }
        }
        
        // Add new list or website
        KeyCode::Char('n') => {
            app.input = Input::default();
            app.input.set_placeholder("New List Name");
            app.mode = TuiMode::Editing;
        }
        KeyCode::Char('a') => {
            if app.selected_list_index.is_some() {
                app.input = Input::default();
                app.input.set_placeholder("New Website URL");
                app.mode = TuiMode::Editing;
            } else {
                app.status_message = "Please select a list first".to_string();
            }
        }
        
        // Delete website or list
        KeyCode::Char('d') => {
            if app.selected_website_index.is_some() {
                app.delete_website();
                app.status_message = "Website removed".to_string();
            }
        }
        KeyCode::Char('D') => {
            if app.selected_list_index.is_some() {
                app.delete_list();
                app.status_message = "List removed".to_string();
            }
        }
        
        _ => {}
    }
    
    Ok(())
}

/// Handle key events for the timer tab
fn handle_timer_tab_events(app: &mut App, key: KeyCode) -> Result<()> {
    match key {
        // Adjust time
        KeyCode::Up => {
            app.increase_time();
        }
        KeyCode::Down => {
            app.decrease_time();
        }
        
        // Change time unit
        KeyCode::Char('t') => {
            app.cycle_time_unit();
        }
        
        // Start blocking
        KeyCode::Enter => {
            if !app.is_blocking && app.selected_list_index.is_some() {
                let websites = app.current_websites();
                
                if !websites.is_empty() {
                    let duration_ms = app.get_blocking_milliseconds();
                    let duration = Duration::from_millis(duration_ms);
                    
                    match start_blocking_websites(&websites, duration_ms) {
                        Ok(_) => {
                            app.start_blocking(duration)?;
                        }
                        Err(e) => {
                            app.status_message = format!("Error blocking websites: {}", e);
                        }
                    }
                } else {
                    app.status_message = "Selected list has no websites to block".to_string();
                }
            }
        }
        
        // Stop blocking
        KeyCode::Esc => {
            if app.is_blocking {
                match stop_blocking_websites() {
                    Ok(_) => {
                        app.stop_blocking()?;
                    }
                    Err(e) => {
                        app.status_message = format!("Error stopping website blocking: {}", e);
                    }
                }
            }
        }
        
        _ => {}
    }
    
    Ok(())
}

/// Block websites using the TUI interface
fn start_blocking_websites(websites: &Vec<String>, _duration_ms: u64) -> std::io::Result<()> {
    let hosts_path = get_hosts_path();
    let config_dir = get_config_dir().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let backup_path = config_dir.join(HOSTS_BACKUP);

    // Read current content of hosts file
    let mut hosts_content = fs::read_to_string(&hosts_path)?;

    // Create backup if it doesn't exist
    if !backup_path.exists() {
        fs::write(&backup_path, &hosts_content)?;
    } else {
        // If backup exists, ensure it's current
        let mut backup_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&backup_path)?;

        backup_file.write_all(hosts_content.as_bytes())?;
    }

    // Remove previous temporary entries if present
    if let Some(start) = hosts_content.find(TEMP_HOSTS_MARKER) {
        if let Some(end) = hosts_content[start..].find("\n# ===== End") {
            hosts_content = hosts_content[..start].to_string() + &hosts_content[start + end + 12..];
        }
    }

    // Add new temporary entries
    hosts_content.push_str(&format!("\n{}\n", TEMP_HOSTS_MARKER));
    for website in websites {
        if !website.trim().is_empty() && !hosts_content.contains(website) {
            hosts_content.push_str(&format!("127.0.0.1\t{}\n", website));
        }
    }
    hosts_content.push_str("# ===== End Temporary Hosts =====\n");

    // Write the updated hosts file
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&hosts_path)?;
    
    file.write_all(hosts_content.as_bytes())?;

    Ok(())
}

/// Stop blocking websites
fn stop_blocking_websites() -> std::io::Result<()> {
    // Same code as in the stop_blocking function
    let hosts_path = get_hosts_path();
    let config_dir = get_config_dir().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let backup_path = config_dir.join(HOSTS_BACKUP);

    if backup_path.exists() {
        let backup_content = fs::read_to_string(&backup_path)?;
        fs::write(&hosts_path, backup_content)?;
    }

    Ok(())
}

/// Stop website blocking and restore hosts file
fn stop_blocking() -> Result<()> {
    let hosts_path = get_hosts_path();
    let config_dir = get_config_dir()?;
    let backup_path = config_dir.join(HOSTS_BACKUP);
    
    if backup_path.exists() {
        let backup_content = fs::read_to_string(&backup_path)?;
        fs::write(&hosts_path, backup_content)?;
    }
    
    Ok(())
}

/// Parse a duration string like "1h", "30m", "45s"
fn parse_duration(duration_str: &str) -> Result<u64> {
    let mut number_str = String::new();
    let mut unit_str = String::new();
    
    for c in duration_str.chars() {
        if c.is_ascii_digit() {
            number_str.push(c);
        } else {
            unit_str.push(c);
        }
    }
    
    let number: u64 = number_str.parse().wrap_err("Invalid duration format")?;
    
    match unit_str.as_str() {
        "s" => Ok(number * 1000),          // seconds to ms
        "m" => Ok(number * 60 * 1000),     // minutes to ms
        "h" => Ok(number * 60 * 60 * 1000),// hours to ms
        _ => Err(color_eyre::eyre::eyre!("Invalid time unit. Use s, m, or h")),
    }
}

/// Application entry point
fn main() -> Result<()> {
    // Setup error handling
    color_eyre::install()?;
    
    let cli = Cli::parse();
    
    match &cli.command {
        Some(Commands::Setup { list_path }) => {
            // Set up the application with a website list
            let _config_dir = get_config_dir()?;
            
            let websites = fs::read_to_string(list_path)
                .wrap_err_with(|| format!("Could not read website list file: {}", list_path))?;
            
            let mut config = load_config()?;
            config.website_list_path = list_path.clone();
            
            // Parse websites and create default lists
            let social_media = tui::WebsiteList {
                name: "Social Media".to_string(),
                websites: vec![
                    "www.facebook.com".to_string(),
                    "facebook.com".to_string(),
                    "www.twitter.com".to_string(),
                    "twitter.com".to_string(),
                    "www.instagram.com".to_string(),
                    "instagram.com".to_string(),
                ],
            };
            
            let entertainment = tui::WebsiteList {
                name: "Entertainment".to_string(),
                websites: vec![
                    "www.youtube.com".to_string(),
                    "youtube.com".to_string(),
                    "www.netflix.com".to_string(),
                    "netflix.com".to_string(),
                    "www.reddit.com".to_string(),
                    "reddit.com".to_string(),
                ],
            };
            
            let user_list = tui::WebsiteList {
                name: "Custom Sites".to_string(),
                websites: websites
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .collect(),
            };
            
            config.website_lists = Some(vec![social_media, entertainment, user_list]);
            save_config(&config)?;
            
            println!("Setup completed successfully!");
        }
        Some(Commands::Reset) => {
            // Reset hosts file to original state
            stop_blocking()?;
            println!("Website blocking has been reset.");
        }
        Some(Commands::Permissions) => {
            // Request permissions
            if check_and_get_permissions()? {
                println!("Required permissions are available.");
            } else {
                println!("Could not obtain required permissions.");
            }
        }
        Some(Commands::Tui) => {
            // TUI application
            run_tui()?;
        }
        None => {
            // CLI mode with direct command
            if let (Some(duration_str), Some(task)) = (&cli.duration, &cli.task) {
                let duration_ms = parse_duration(duration_str)?;
                let duration = Duration::from_millis(duration_ms);
                
                let config = load_config()?;
                let mut websites = Vec::new();
                
                if let Some(website_lists) = &config.website_lists {
                    for list in website_lists {
                        websites.extend(list.websites.clone());
                    }
                } else {
                    // Try to read from website list path
                    let website_list = fs::read_to_string(&config.website_list_path)
                        .wrap_err_with(|| format!("Could not read website list: {}", &config.website_list_path))?;
                    
                    websites = website_list
                        .lines()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty() && !s.starts_with('#'))
                        .collect();
                }
                
                if websites.is_empty() {
                    println!("No websites to block. Please set up the application first.");
                    return Ok(());
                }
                
                block_websites_with_timer(&websites, duration, task, duration_str)?;
            } else {
                // Show usage info
                let supported_commands = [
                    "tui                - Start the TUI interface",
                    "setup --list <path>- Set up website lists from file",
                    "reset              - Reset all website blocking",
                    "permissions        - Check/request required permissions",
                    "-d <time> -t <task>- Block websites for duration (e.g., -d 30m -t work)",
                ];
                
                println!("TimeGuardian - Focus by blocking distracting websites");
                println!("Created by: Jannis Krija (https://github.com/cipher-shad0w)\n");
                println!("Usage: timeguardian [COMMAND] [OPTIONS]\n");
                println!("Commands:");
                for cmd in supported_commands {
                    println!("  {}", cmd);
                }
                println!("\nTime units: s (seconds), m (minutes), h (hours)");
            }
        }
    }
    
    Ok(())
}
