<div align="center" id="top"> 
  &#xa0;
</div>

<h1 align="center">TimeGuardian</h1>

<p align="center">
  <img alt="Github top language" src="https://img.shields.io/github/languages/top/cipher-shad0w/timeguardian?color=56BEB8">
  <img alt="Github language count" src="https://img.shields.io/github/languages/count/cipher-shad0w/timeguardian?color=56BEB8">
  <img alt="Repository size" src="https://img.shields.io/github/repo-size/cipher-shad0w/timeguardian?color=56BEB8">
</p>

<p align="center">
  <a href="#about">About</a> &#xa0; | &#xa0; 
  <a href="#features">Features</a> &#xa0; | &#xa0;
  <a href="#technologies">Technologies</a> &#xa0; | &#xa0;
  <a href="#requirements">Requirements</a> &#xa0; | &#xa0;
  <a href="#setup">Setup</a> &#xa0; | &#xa0;
  <a href="#usage">Usage</a> &#xa0; | &#xa0;
  <a href="#structure">Project Structure</a> &#xa0; | &#xa0;
  <a href="#license">License</a>
</p>

---

## <span id="about"></span> :dart: About

TimeGuardian is a command-line and TUI (Text User Interface) application designed to help you maintain focus by blocking distracting websites. It provides an easy way to manage lists of websites to block during focused work sessions, with support for configurable timers to help implement productivity techniques like Pomodoro.

---

## <span id="features"></span> :star: Features

- Block distracting websites with configurable website lists
- Create multiple website block lists for different scenarios
- TUI for easy management and visualization 
- Configurable focus timers with Pomodoro technique support
- Command-line interface for quick blocking and unblocking
- Cross-platform support (Linux, macOS, Windows)
- Automatic host file management

---

## <span id="technologies"></span> :rocket: Technologies

- **Programming Language:** Rust
- **Main Libraries:**
  - `ratatui`: Terminal user interface framework
  - `crossterm`: Terminal manipulation
  - `clap`: Command-line argument parsing
  - `serde`: Data serialization and deserialization
  - `toml`: Configuration file format
  - `color-eyre`: Error handling

---

## <span id="requirements"></span> :white_check_mark: Requirements

- [Git](https://git-scm.com) (for cloning the repository)
- [Rust](https://www.rust-lang.org/tools/install) (1.70.0 or higher)
- Administrative/sudo privileges (for modifying host files)

---

## <span id="setup"></span> :checkered_flag: Setup

1. **Clone the repository:**
   ```
   git clone https://github.com/cipher-shad0w/timeguardian.git
   cd timeguardian
   ```

2. **Build the application:**
   ```
   cargo build --release
   ```

3. **Install the application (optional):**
   ```
   cargo install --path .
   ```

---

## <span id="usage"></span> :computer: Usage

### TUI Mode

Launch the TUI interface:
```
timeguardian
```

The TUI provides the following features:
- Create and manage website block lists
- Set up focus timers
- View current blocking status
- Toggle blocking for specific website lists

### Command-line Mode

Block websites from a specific list:
```
timeguardian block <list-name>
```

Unblock websites:
```
timeguardian unblock
```

Start a focus timer with website blocking:
```
timeguardian focus --list <list-name> --minutes 25
```

List all available website lists:
```
timeguardian lists
```

---

## <span id="structure"></span> :file_folder: Project Structure

```
LICENSE
Cargo.toml
README.md
.cross.toml
.gitignore

src/
    main.rs
    tui/
        app.rs
        event.rs
        mod.rs
        ui.rs
```

- `main.rs`: Application entry point
- `tui/`: Text User Interface implementation
- `Cargo.toml`: Rust dependencies and project metadata

---

## <span id="license"></span> :memo: License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

---

## Author

Made with :heart: by [Jannis Krija](https://github.com/cipher-shad0w)

&#xa0;

<a href="#top">Back to top</a>