/*
* TimeGuardian TUI Module
* Author: Jannis Krija (https://github.com/cipher-shad0w)
* 
* This is the root module for the TUI (Text User Interface) components.
* It re-exports all submodules and their public items for easier access.
*/

pub mod app;
pub mod event;
pub mod ui;

// Re-export the main App struct and TuiMode for convenience
pub use app::{App, TuiMode, WebsiteList};