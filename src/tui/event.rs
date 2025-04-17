/*
* TimeGuardian TUI Event Module
* Author: Jannis Krija (https://github.com/cipher-shad0w)
* 
* This module handles events for the TUI, including keyboard input and timed events.
* It uses a multi-producer, single-consumer channel to handle events asynchronously.
*/

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// Terminal events
pub enum Event {
    /// Key event from keyboard
    Key(KeyEvent),
    /// Mouse event (reserved for future use)
    Mouse(MouseEvent),
    /// Terminal resize event (reserved for future use)
    Resize(u16, u16),
    /// Tick event for UI refresh
    Tick,
}

/// Event handler for processing terminal events
pub struct EventHandler {
    /// Event receiver channel
    pub receiver: mpsc::Receiver<Event>,
    #[allow(dead_code)]
    sender: mpsc::Sender<Event>,
}

impl EventHandler {
    /// Create a new event handler with specified tick rate
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let event_sender = sender.clone();
        
        // Spawn a thread that handles events
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            
            loop {
                // Calculate timeout for the next tick
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(Duration::from_secs(0));
                
                // Check for events with the calculated timeout
                if event::poll(timeout).unwrap() {
                    match event::read().unwrap() {
                        CrosstermEvent::Key(key) => {
                            if let Err(err) = event_sender.send(Event::Key(key)) {
                                eprintln!("Error sending key event: {:?}", err);
                                // Most likely the channel has been closed, so exit the thread
                                return;
                            }
                        }
                        CrosstermEvent::Mouse(mouse) => {
                            if let Err(err) = event_sender.send(Event::Mouse(mouse)) {
                                eprintln!("Error sending mouse event: {:?}", err);
                                return;
                            }
                        }
                        CrosstermEvent::Resize(width, height) => {
                            if let Err(err) = event_sender.send(Event::Resize(width, height)) {
                                eprintln!("Error sending resize event: {:?}", err);
                                return;
                            }
                        }
                        // Ignoring FocusGained and FocusLost events
                        _ => {}
                    }
                }
                
                // Check if tick rate has elapsed and send a Tick event
                if last_tick.elapsed() >= tick_rate {
                    // Reset the last tick time
                    last_tick = Instant::now();
                    
                    // Send tick event
                    if let Err(err) = event_sender.send(Event::Tick) {
                        eprintln!("Error sending tick event: {:?}", err);
                        return;
                    }
                }
            }
        });
        
        Self { receiver, sender }
    }
}