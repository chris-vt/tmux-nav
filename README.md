# tmux-nav

A lightning-fast, native-feeling directory tree sidebar for Tmux, written in Rust.

`tmux-nav` tightly integrates a file system explorer into your Tmux workflow. It spawns as a seamless sidebar pane and uses two-way Inter-Process Communication (IPC) to stay perfectly synchronized with your shell.

## Features

- **Two-Way Synchronization**: 
  - **Inbound**: When you `cd` in your shell, the tree automatically updates its root to match your current directory.
  - **Outbound**: When you press `Enter` on a directory in the tree, your companion shell instantly `cd`s into it.
- **Smart File Launching**: Pressing `Enter` on a text file automatically commands the companion pane to open it in Neovim (`nvim`).
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

To get the intended "seamless sidebar" experience, you must configure both your shell and Tmux.

### 1. Zsh Configuration
For `tmux-nav` to track your current directory, you must add its IPC hook to your `~/.zshrc` (or equivalent Zsh configuration file). This hook safely broadcasts directory changes over a Unix socket whenever you are inside a Tmux pane.

Add the following function to your Zsh configuration:

```bash
# tmux-nav hook
chpwd() {
    if [[ -n "$TMUX_PANE" ]]; then
        local sock="/tmp/tmux_nav_${TMUX_PANE}.sock"
        if [[ -S "$sock" ]]; then
            echo "$PWD" | nc -U -N "$sock" >/dev/null 2>&1 &!
        fi
    fi
}
```

### 2. Tmux Configuration
You need to map keybindings to launch the app and toggle features. Add the following to your `tmux.conf`:

```tmux
# Launch the sidebar (using the bundled wrapper script)
bind -n C-Up run-shell "tmux-nav-split"

# Quickly jump back and forth between the tree and shell pane
bind -n C-o select-pane -t :.+

# Toggle hidden files in the tree from either pane
bind -n C-p run-shell "tmux-nav --toggle-hidden #{pane_id}"
```

## Usage

Hit your configured Tmux bind (e.g., `Ctrl + Up`). A new window will open with `tmux-nav` taking up 15% of the left side of your screen. 

<<<<<<< HEAD
Your cursor focus will be placed in the shell on the right, but you can jump back to the tree sidebar at any time using your standard Tmux pane navigation keys.
=======
Your cursor focus will be placed in the shell on the right, but you can jump back to the tree sidebar at any time using your standard Tmux pane navigation keys (or your `Ctrl + o` bind).
>>>>>>> dev

**Keyboard Controls in `tmux-nav`**:
- `Up` / `Down`: Navigate the file tree
- `Right`: Expand a directory
- `Left`: Collapse a directory
<<<<<<< HEAD
- `Enter`: Command the companion shell to change directory into the currently selected folder
- `Ctrl + C`: Exit
=======
- `Enter`: 
  - On a **folder**: Commands the companion shell to `cd` into it
  - On a **file**: Commands the companion shell to open it in `nvim`
- `Ctrl + p`: Toggle hidden files (dotfiles)
- `Ctrl + c`: Exit
>>>>>>> dev
