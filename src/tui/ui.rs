/*
* TimeGuardian TUI UI Module
* Author: Jannis Krija (https://github.com/cipher-shad0w)
* 
* This module handles the UI rendering for the TimeGuardian TUI.
* It defines layout, widgets, and drawing functions for the interface.
*/

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap,
    },
    Frame,
};

use crate::tui::{App, TuiMode};

/// Time unit enum for the timer tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    /// Minutes (default)
    Minutes,
    /// Hours
    Hours,
    /// Seconds
    Seconds,
}

/// Tab state for managing tab navigation
pub struct TabsState {
    /// List of tab titles
    pub titles: Vec<&'static str>,
    /// Index of the currently selected tab
    pub index: usize,
}

impl TabsState {
    /// Create a new tabs state with the given titles
    pub fn new(titles: Vec<&'static str>) -> Self {
        Self { titles, index: 0 }
    }
    
    /// Select the next tab
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }
    
    /// Select the previous tab
    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

/// Main render function for the UI
pub fn render(app: &mut App, frame: &mut Frame) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title bar
            Constraint::Min(0),     // Main area
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.size());
    
    // Render the title and tabs
    render_title_and_tabs(app, frame, chunks[0]);
    
    // Render the content based on the selected tab
    match app.tabs.index {
        0 => render_website_lists_tab(app, frame, chunks[1]),
        1 => render_timer_tab(app, frame, chunks[1]),
        _ => {}
    }
    
    // Render the status bar
    render_status_bar(app, frame, chunks[2]);
    
    // Render help popup if in help mode
    if app.mode == TuiMode::Help {
        render_help_popup(app, frame);
    }
}

/// Render the title bar and tabs
fn render_title_and_tabs(app: &App, frame: &mut Frame, area: Rect) {
    // Create title spans
    let title_spans = vec![
        Span::styled("Time", Style::default().fg(Color::Green)),
        Span::styled("Guardian", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::raw(" - Block distractions, stay focused"),
    ];
    
    let title = Paragraph::new(Line::from(title_spans))
        .style(Style::default())
        .block(Block::default().borders(Borders::BOTTOM));
    
    // Create tabs
    let tab_titles: Vec<Line> = app
        .tabs
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default()),
            ])
        })
        .collect();
    
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::NONE))
        .select(app.tabs.index)
        .style(Style::default())
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    
    // Split the area for title and tabs
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);
    
    frame.render_widget(title, chunks[0]);
    frame.render_widget(tabs, chunks[1]);
}

/// Render the website lists tab
fn render_website_lists_tab(app: &mut App, frame: &mut Frame, area: Rect) {
    // Split the area into two columns for lists and websites
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(area);
    
    // Render the list of website lists
    let lists_block = Block::default()
        .title("Website Lists")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    
    let list_items: Vec<ListItem> = app
        .website_lists
        .iter()
        .map(|list| {
            let lines = vec![Line::from(vec![Span::styled(
                &list.name,
                Style::default().fg(Color::White),
            )])];
            ListItem::new(lines)
        })
        .collect();
    
    let lists = List::new(list_items)
        .block(lists_block)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    frame.render_stateful_widget(lists, chunks[0], &mut app.website_list_state);
    
    // Render the websites in the selected list
    let websites_title = if let Some(index) = app.selected_list_index {
        if index < app.website_lists.len() {
            format!("Websites in {}", app.website_lists[index].name)
        } else {
            "Websites".to_string()
        }
    } else {
        "Websites".to_string()
    };
    
    let websites_block = Block::default()
        .title(websites_title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    
    // Get websites from selected list
    let website_items: Vec<ListItem> = if let Some(list) = app.current_website_list() {
        list.websites
            .iter()
            .map(|website| {
                let lines = vec![Line::from(Span::raw(website))];
                ListItem::new(lines)
            })
            .collect()
    } else {
        Vec::new()
    };
    
    let websites = List::new(website_items)
        .block(websites_block)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    
    frame.render_stateful_widget(websites, chunks[1], &mut app.website_state);
    
    // Render input box if in editing mode
    if app.mode == TuiMode::Editing && app.tabs.index == 0 {
        render_input_box(app, frame);
    }
}

/// Render the timer tab
fn render_timer_tab(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Timer controls
            Constraint::Length(3),  // Selected list
            Constraint::Min(0),     // Timer status
        ])
        .split(area);
    
    // Timer display and controls
    let unit_display = match app.time_unit {
        TimeUnit::Minutes => "minutes",
        TimeUnit::Hours => "hours",
        TimeUnit::Seconds => "seconds",
    };
    
    let timer_text = if app.is_blocking {
        if let Some(remaining) = app.get_remaining_time() {
            format!(
                "Blocking websites... Time remaining: {}",
                app.format_duration(remaining)
            )
        } else {
            "Blocking websites...".to_string()
        }
    } else {
        format!("Block for {} {}", app.time_value, unit_display)
    };
    
    let timer_block = Block::default()
        .title("Timer Settings")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    
    let timer_paragraph = Paragraph::new(timer_text)
        .block(timer_block)
        .style(if app.is_blocking {
            Style::default().fg(Color::Red)
        } else {
            Style::default()
        });
    
    frame.render_widget(timer_paragraph, chunks[0]);
    
    // Selected list info
    let selected_list_info = if let Some(index) = app.selected_list_index {
        if index < app.website_lists.len() {
            let list = &app.website_lists[index];
            format!(
                "Selected list: {} ({} websites)",
                list.name,
                list.websites.len()
            )
        } else {
            "No list selected".to_string()
        }
    } else {
        "No list selected".to_string()
    };
    
    let list_block = Block::default()
        .title("List Info")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    
    let list_paragraph = Paragraph::new(selected_list_info).block(list_block);
    frame.render_widget(list_paragraph, chunks[1]);
    
    // Help text
    let help_text = if app.is_blocking {
        "Press [Esc] to stop blocking"
    } else {
        "Press [↑/↓] to adjust time | [t] to change unit | [Enter] to start blocking"
    };
    
    let instructions = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("Instructions")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Yellow));
    
    frame.render_widget(instructions, chunks[2]);
}

/// Render the status bar
fn render_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    // Create the status message with mode indicator
    let mode_indicator = match app.mode {
        TuiMode::Normal => "[Normal]",
        TuiMode::Editing => "[Editing]",
        TuiMode::Help => "[Help]",
    };
    
    let status = format!("{} {}", mode_indicator, app.status_message);
    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(Color::White).bg(Color::Blue));
    
    frame.render_widget(status_bar, area);
}

/// Render the input box for editing
fn render_input_box(app: &App, frame: &mut Frame) {
    // Create a centered popup for the input
    let area = centered_rect(60, 3, frame.size());
    
    // Render the input popup
    let input_block = Block::default()
        .title("Input")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));
    
    let input_widget = Paragraph::new(app.input.value())
        .style(Style::default())
        .block(input_block);
    
    // Render a background to create a popup effect
    frame.render_widget(Clear, area);
    frame.render_widget(input_widget, area);
    
    // Set cursor position
    frame.set_cursor(
        area.x + app.input.visual_cursor() as u16 + 1,
        area.y + 1,
    );
}

/// Render the help popup
fn render_help_popup(app: &App, frame: &mut Frame) {
    let area = centered_rect(70, 20, frame.size());
    
    // Clear the area
    frame.render_widget(Clear, area);
    
    // Create the help text based on the current tab
    let help_text = match app.tabs.index {
        0 => get_website_lists_tab_help(),
        1 => get_timer_tab_help(),
        _ => Vec::new(),
    };
    
    let help_block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    
    let help_paragraph = Paragraph::new(Text::from(help_text))
        .block(help_block)
        .wrap(Wrap { trim: true });
    
    frame.render_widget(help_paragraph, area);
}

/// Get help text for the website lists tab
fn get_website_lists_tab_help() -> Vec<Line> {
    vec![
        Line::from(vec![
            Span::styled("Website Lists Tab", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  [Tab] / [Shift+Tab]: Switch between tabs"),
        Line::from("  [←/→]: Switch between lists and websites"),
        Line::from("  [↑/↓]: Navigate within lists or websites"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  [n]: Create a new website list"),
        Line::from("  [a]: Add a website to the selected list"),
        Line::from("  [d]: Delete selected website"),
        Line::from("  [D]: Delete selected list"),
        Line::from(""),
        Line::from("Other:"),
        Line::from("  [?]: Toggle help"),
        Line::from("  [q]: Quit application"),
    ]
}

/// Get help text for the timer tab
fn get_timer_tab_help() -> Vec<Line> {
    vec![
        Line::from(vec![
            Span::styled("Timer Tab", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("Timer Controls:"),
        Line::from("  [↑/↓]: Increase/decrease time"),
        Line::from("  [t]: Change time unit (minutes, hours, seconds)"),
        Line::from("  [Enter]: Start blocking websites"),
        Line::from("  [Esc]: Stop active blocking session"),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  [Tab] / [Shift+Tab]: Switch between tabs"),
        Line::from(""),
        Line::from("Other:"),
        Line::from("  [?]: Toggle help"),
        Line::from("  [q]: Quit application"),
        Line::from(""),
        Line::from("Note: You must select a website list in the Website Lists tab first"),
    ]
}

/// Create a centered rectangle for popups
fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_width = (r.width * percent_x) / 100;
    
    let popup_x = (r.width - popup_width) / 2;
    let popup_y = (r.height - height) / 2;
    
    Rect::new(
        r.x + popup_x,
        r.y + popup_y,
        popup_width,
        height,
    )
}