/*
* TimeGuardian TUI App Module
* Author: Jannis Krija (https://github.com/cipher-shad0w)
* 
* This module contains the core application state and logic for the TimeGuardian TUI.
* It manages website lists, blocking sessions, and user interactions.
*/

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tui_input::Input;

use crate::tui::{
    ui::{TabsState, TimeUnit},
};

/// Result type for app operations
pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Website list structure 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebsiteList {
    pub name: String,
    pub websites: Vec<String>,
}

/// Application mode enum for the UI state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiMode {
    /// Normal mode for navigation
    Normal,
    /// Editing mode for text input
    Editing,
    /// Help screen mode
    Help,
}

/// Main application state structure
pub struct App {
    /// Whether the application is still running
    pub running: bool,
    
    /// Current tab state
    pub tabs: TabsState,
    
    /// User input field
    pub input: Input,
    
    /// Current UI mode
    pub mode: TuiMode,
    
    /// Status message to display to the user
    pub status_message: String,
    
    /// List of website lists
    pub website_lists: Vec<WebsiteList>,
    
    /// Selected website list index
    pub selected_list_index: Option<usize>,
    
    /// Selected website index 
    pub selected_website_index: Option<usize>,
    
    /// Website list state for UI rendering
    pub website_list_state: ratatui::widgets::ListState,
    
    /// Website state for UI rendering
    pub website_state: ratatui::widgets::ListState,
    
    /// Whether the application is currently blocking websites
    pub is_blocking: bool,
    
    /// Time when the current blocking session ends
    pub blocking_end_time: Option<Instant>,
    
    /// Duration of the current blocking session
    pub block_duration_ms: u64,
    
    /// Time unit for the timer tab
    pub time_unit: TimeUnit,
    
    /// Time value for the timer tab
    pub time_value: u64,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        Self {
            running: true,
            tabs: TabsState::new(vec!["Website Lists", "Timer"]),
            input: Input::default(),
            mode: TuiMode::Normal,
            status_message: String::new(),
            website_lists: Vec::new(),
            selected_list_index: None,
            selected_website_index: None,
            website_list_state: ratatui::widgets::ListState::default(),
            website_state: ratatui::widgets::ListState::default(),
            is_blocking: false,
            blocking_end_time: None,
            block_duration_ms: 25 * 60 * 1000, // Default: 25 minutes
            time_unit: TimeUnit::Minutes,
            time_value: 25,
        }
    }
    
    /// Initialize the application
    pub fn init(&mut self) -> Result<()> {
        self.status_message = "Welcome to TimeGuardian! Press '?' for help.".to_string();
        Ok(())
    }
    
    /// Get the websites from the currently selected list
    pub fn current_websites(&self) -> Vec<String> {
        if let Some(index) = self.selected_list_index {
            if index < self.website_lists.len() {
                return self.website_lists[index].websites.clone();
            }
        }
        Vec::new()
    }
    
    /// Get the currently selected website list 
    pub fn current_website_list(&self) -> Option<&WebsiteList> {
        if let Some(index) = self.selected_list_index {
            if index < self.website_lists.len() {
                return Some(&self.website_lists[index]);
            }
        }
        None
    }
    
    /// Add a new website to the selected list
    pub fn add_website(&mut self, website: String) {
        if let Some(index) = self.selected_list_index {
            if index < self.website_lists.len() {
                let cleaned_website = website.trim().to_string();
                if !cleaned_website.is_empty() {
                    let list = &mut self.website_lists[index];
                    
                    // Skip if already exists
                    if !list.websites.contains(&cleaned_website) {
                        list.websites.push(cleaned_website);
                        
                        // Auto select the new website
                        let new_index = list.websites.len() - 1;
                        self.website_state.select(Some(new_index));
                        self.selected_website_index = Some(new_index);
                    }
                }
            }
        }
    }
    
    /// Delete the selected website
    pub fn delete_website(&mut self) {
        if let (Some(list_index), Some(website_index)) = (self.selected_list_index, self.selected_website_index) {
            if list_index < self.website_lists.len() {
                let list = &mut self.website_lists[list_index];
                if website_index < list.websites.len() {
                    list.websites.remove(website_index);
                    
                    // Update selection
                    if list.websites.is_empty() {
                        self.website_state.select(None);
                        self.selected_website_index = None;
                    } else {
                        let new_index = if website_index >= list.websites.len() {
                            list.websites.len() - 1
                        } else {
                            website_index
                        };
                        self.website_state.select(Some(new_index));
                        self.selected_website_index = Some(new_index);
                    }
                }
            }
        }
    }
    
    /// Add a new website list
    pub fn add_list(&mut self, name: String) {
        let cleaned_name = name.trim().to_string();
        if !cleaned_name.is_empty() {
            // Skip if name already exists
            if !self.website_lists.iter().any(|list| list.name == cleaned_name) {
                self.website_lists.push(WebsiteList {
                    name: cleaned_name,
                    websites: Vec::new(),
                });
                
                // Auto select the new list
                let new_index = self.website_lists.len() - 1;
                self.website_list_state.select(Some(new_index));
                self.selected_list_index = Some(new_index);
                
                // Clear website selection
                self.website_state.select(None);
                self.selected_website_index = None;
            }
        }
    }
    
    /// Delete the selected website list
    pub fn delete_list(&mut self) {
        if let Some(index) = self.selected_list_index {
            if index < self.website_lists.len() {
                self.website_lists.remove(index);
                
                // Update selection
                if self.website_lists.is_empty() {
                    self.website_list_state.select(None);
                    self.selected_list_index = None;
                } else {
                    let new_index = if index >= self.website_lists.len() {
                        self.website_lists.len() - 1
                    } else {
                        index
                    };
                    self.website_list_state.select(Some(new_index));
                    self.selected_list_index = Some(new_index);
                }
                
                // Clear website selection
                self.website_state.select(None);
                self.selected_website_index = None;
            }
        }
    }
    
    /// Process a tick event
    pub fn tick(&mut self) {
        // Update any time-based state here
    }
    
    /// Increase the blocking time value
    pub fn increase_time(&mut self) {
        match self.time_unit {
            TimeUnit::Minutes => {
                if self.time_value < 120 {
                    self.time_value += 5;
                }
            }
            TimeUnit::Hours => {
                if self.time_value < 8 {
                    self.time_value += 1;
                }
            }
            TimeUnit::Seconds => {
                if self.time_value < 55 {
                    self.time_value += 5;
                } else {
                    self.time_value = 60;
                }
            }
        }
        self.update_blocking_duration();
    }
    
    /// Decrease the blocking time value
    pub fn decrease_time(&mut self) {
        match self.time_unit {
            TimeUnit::Minutes => {
                if self.time_value > 5 {
                    self.time_value -= 5;
                } else {
                    self.time_value = 1;
                }
            }
            TimeUnit::Hours => {
                if self.time_value > 1 {
                    self.time_value -= 1;
                }
            }
            TimeUnit::Seconds => {
                if self.time_value > 10 {
                    self.time_value -= 5;
                } else {
                    self.time_value = 5;
                }
            }
        }
        self.update_blocking_duration();
    }
    
    /// Cycle through available time units
    pub fn cycle_time_unit(&mut self) {
        match self.time_unit {
            TimeUnit::Minutes => {
                self.time_unit = TimeUnit::Hours;
                self.time_value = 1; // Default hour value
            }
            TimeUnit::Hours => {
                self.time_unit = TimeUnit::Seconds;
                self.time_value = 30; // Default seconds value
            }
            TimeUnit::Seconds => {
                self.time_unit = TimeUnit::Minutes;
                self.time_value = 25; // Default minute value
            }
        }
        self.update_blocking_duration();
    }
    
    /// Update the blocking duration based on the current time unit and value
    fn update_blocking_duration(&mut self) {
        match self.time_unit {
            TimeUnit::Minutes => {
                self.block_duration_ms = self.time_value * 60 * 1000;
            }
            TimeUnit::Hours => {
                self.block_duration_ms = self.time_value * 60 * 60 * 1000;
            }
            TimeUnit::Seconds => {
                self.block_duration_ms = self.time_value * 1000;
            }
        }
    }
    
    /// Get the blocking duration in milliseconds
    pub fn get_blocking_milliseconds(&self) -> u64 {
        self.block_duration_ms
    }
    
    /// Start a blocking session
    pub fn start_blocking(&mut self, duration: Duration) -> Result<()> {
        self.is_blocking = true;
        self.blocking_end_time = Some(Instant::now() + duration);
        self.status_message = format!(
            "Blocking websites for {:?}",
            self.format_duration(duration)
        );
        Ok(())
    }
    
    /// Stop the current blocking session
    pub fn stop_blocking(&mut self) -> Result<()> {
        self.is_blocking = false;
        self.blocking_end_time = None;
        self.status_message = "Website blocking stopped".to_string();
        Ok(())
    }
    
    /// Format a duration for display
    pub fn format_duration(&self, duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        
        if hours > 0 {
            format!("{}h {:02}m {:02}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {:02}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
    
    /// Get the remaining time in the current blocking session
    pub fn get_remaining_time(&self) -> Option<Duration> {
        if self.is_blocking {
            if let Some(end_time) = self.blocking_end_time {
                let now = Instant::now();
                if now < end_time {
                    return Some(end_time - now);
                }
            }
        }
        None
    }
    
    /// Save configuration to file (unused but kept for future functionality)
    pub fn save_configuration(&mut self) -> AppResult<()> {
        // Save configuration logic would go here
        Ok(())
    }
}