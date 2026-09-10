# tmux-nav

A lightning-fast, native-feeling directory tree sidebar for Tmux, written in Rust.

`tmux-nav` tightly integrates a file system explorer into your Tmux workflow. It spawns as a seamless sidebar pane and uses two-way Inter-Process Communication (IPC) to stay perfectly synchronized with your shell.

## Features

- **Two-Way Synchronization**: 
  - **Inbound**: When you `cd` in your shell, the tree automatically updates its root to match your current directory.
  - **Outbound**: When you press `Enter` on a directory in the tree, your companion shell instantly `cd`s into it.
- **Native Aesthetic**: The bundled integration script completely hides Tmux pane borders, making the sidebar feel like a native application feature rather than a split terminal.
- **Minimalist Icons**: Uses standard Nerd Font icons (``, ``, ``) to match the clean aesthetic of tools like `eza`.
- **Zero Input Lag**: Built in Rust with `ratatui` and relies on raw Unix Sockets for instantaneous state updates.

## Prerequisites

- **Nix** (Flakes enabled)
- **Tmux**
- **Zsh** (The inbound IPC hook is currently written for `zsh`'s `chpwd` function)
- A terminal patched with a **Nerd Font**

## Installation (NixOS / Home Manager)

Because `tmux-nav` is packaged as a Nix flake, it's trivial to add to your declarative system configuration.

1. Add the repository to your flake `inputs`:
```nix
inputs = {
  # ... your other inputs
  tmux-nav.url = "github:chris-vt/tmux-nav"; # Update if your github handle differs
};
```

2. Add the package to your system or user packages (ensure you pass `inputs` to your modules):
```nix
environment.systemPackages = [
  inputs.tmux-nav.packages.${pkgs.system}.default
];
```

## Setup & Configuration

To get the intended "seamless sidebar" experience, you need to map a Tmux keybinding to the bundled wrapper script (`tmux-nav-split`), which handles spawning the window, hiding the borders, and hooking up the IPC.

Add the following to your `tmux.conf` (e.g., mapping to `Ctrl + Up`):

```tmux
bind -n C-Up run-shell "tmux-nav-split"
```

*(Note: The `tmux-nav-split` script automatically injects the necessary Zsh IPC hook into the newly created shell pane. If you ever want the hook globally active across all terminals, you can add `eval "$(tmux-nav --init-zsh)"` to your `~/.zshrc`.)*

## Usage

Hit your configured Tmux bind (e.g., `Ctrl + Up`). A new window will open with `tmux-nav` taking up 15% of the left side of your screen. 

Your cursor focus will be placed in the shell on the right, but you can jump back to the tree sidebar at any time using your standard Tmux pane navigation keys.

**Keyboard Controls in `tmux-nav`**:
- `Up` / `Down`: Navigate the file tree
- `Right`: Expand a directory
- `Left`: Collapse a directory
- `Enter`: Command the companion shell to change directory into the currently selected folder
- `Ctrl + C`: Exit
