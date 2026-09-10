## Project Specification: `tmux-nav` (Bidirectional Terminal Tree Navigator)

### 1. Objective

A standalone, high-performance TUI companion application running inside a dedicated `tmux` sidebar pane. It provides a real-time, interactive, click-responsive directory tree that stays bidirectionally synchronized with an adjacent interactive shell (`zsh`) session. Supports multiple independent instances running in different tmux windows/panes.

---

### 2. Operational Architecture

```
 ┌─────────────────────────────────────────┐        ┌───────────────────────────────┐
 │          Sidebar Pane (Left)            │        │      Shell Pane (Right)       │
 │                                         │        │                               │
 │       tmux-nav (Rust TUI binary)        │        │   Interactive zsh session     │
 │                                         │        │                               │
 │ ┌─────────────┐ ┌─────────────────────┐ │        │                               │
 │ │ Tree (30%)  │ │ Preview/Info (70%)  │ │        │   • chpwd hook                │
 │ │             │ │                     │ │        │   • Standard CLI commands     │
 │ │ • crossterm │ │ • eza-like UI       │ │        │                               │
 │ │ • ratatui   │ │                     │ │        │                               │
 │ └─────────────┘ └─────────────────────┘ │        │                               │
 └────────────────────┬────────────────────┘        └───────────────┬───────────────┘
                      │                                             │
                      │ 1. Inbound sync: reads $PWD on chpwd        │
                      │◄────────────────────────────────────────────┤  (UNIX Domain Socket)
                      │                                             │
                      │ 2. Outbound sync: sends `cd '<path>'\n`     │
                      ├────────────────────────────────────────────►│  (tmux send-keys)
                      │                                             │
```

---

### 3. Functional Requirements

#### F1. Real-Time Viewport & UI Layout

* **Layout**: Two dedicated split panels. The left panel (approx 30% horizontal space) displays the directory tree. The right panel (70%) serves as a preview area.
* **Look and Feel**: Visuals should be heavily inspired by `eza` (icons, colors, clean formatting).
* **Tree View**: By default, displays only the immediate folders and files in the directory pointed to by the console.
* Anchor the current directory view strictly at Row 1 (top of the pane), eliminating downward terminal scroll overflow regardless of directory item count.
* Maintain an active filesystem watcher (`notify` crate) to update items dynamically when files are added, removed, or renamed on disk.

#### F2. Input Handling & Interactivity

* **Mouse Interactions** (via `crossterm` mouse tracking):
  * **Single-Click (Tree)**: Expand/collapse folder contents in a tree-like style. Select and highlight an item to immediately display folder contents or file metadata in the preview panel.
  * **Double-Click (Directory)**: Navigate the companion console into the selected folder (outbound sync).
  * **Double-Click (Parent `..`) / Right-Click**: Ascend to the parent directory.
  * **Scroll Wheel**: Smoothly scroll long directory listings without switching into `tmux` copy-mode.

* **Keyboard Navigation**:
  * Arrow keys / Vim bindings (`j`/`k` for vertical movement, `l` or `Enter` to descend/expand, `h` to ascend/collapse).

#### F3. Bidirectional IPC Synchronization

* **Instance Targeting**: The target `tmux` pane ID (e.g., `%1`) is passed as a command-line argument when launching `tmux-nav` (e.g., `tmux-nav --target-pane %1`), allowing multiple robust instances to run independently.
* **Zsh Hook Integration (`tmux-nav init-zsh`)**: The app provides a command that generates a shell hook to be evaluated in `.zshrc`. The generated hook predictably uses the shell's `$TMUX_PANE` variable to determine the correct UNIX socket path.
* **Inbound Synchronization (Shell $\rightarrow$ Navigator)**:
  * Listen on a designated UNIX domain socket (e.g., `/tmp/tmux_nav_${TARGET_PANE}.sock`).
  * On shell directory changes (`chpwd`), the active path is sent across the socket; the navigator updates its root view to match without polling delays.
* **Outbound Synchronization (Navigator $\rightarrow$ Shell)**:
  * When a directory transition is triggered within the TUI (via double-click or keyboard), dispatch `tmux send-keys -t <target-pane> "cd '<path>'\n"`.
* **State Loop Suppression**:
  * The TUI must track its current active path and suppress outbound dispatch if an inbound socket message reports an identical path.

#### F4. Lifecycle Management

* **Auto-Exit**: The `tmux-nav` application must automatically exit to clean up its pane if it detects that its companion target shell pane has been closed or destroyed.

---

### 4. Non-Functional Requirements

* **Performance**: Sub-10ms response time on keypresses and mouse clicks; zero idle CPU polling (event-driven via `epoll`/`inotify` and socket event channels).
* **Self-Contained Binary**: Written in Rust with no dynamic external library dependencies; cleanly distributable via a single Nix flake package derivation.
* **Resilience**: Clean shutdown traps to ensure UNIX socket files and mouse capture modes (`DisableMouseCapture`) are dereferenced on exit, leaving the terminal emulator in a clean state.

---

### 5. Proposed Technology Stack

| Layer | Component / Library | Purpose |
| --- | --- | --- |
| **Language** | Rust (2021 edition) | Memory safety, zero runtime, native NixOS integration |
| **TUI & Input** | `ratatui` + `crossterm` | Terminal drawing, mouse tracking, raw mode management |
| **IPC (Inbound)** | `std::os::unix::net::UnixListener` | Non-blocking socket reader for `zsh` path broadcasts |
| **IPC (Outbound)** | `std::process::Command` (`tmux`) | Pane targeting and direct command dispatch |
| **FS Events** | `notify` crate | Inotify kernel events for reactive live file tracking |