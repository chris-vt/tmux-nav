#!/usr/bin/env bash
W_ID=$(tmux new-window -P -F "#{window_id}")
P_SHELL=$(tmux display-message -t "$W_ID.0" -p "#{pane_id}")
P_NAV=$(tmux split-window -t "$W_ID" -h -l 15% -b -P -F "#{pane_id}")

# Start tmux-nav in the left pane
tmux send-keys -t "$P_NAV" "cargo run --manifest-path=/home/chris/03.projects/marlino/Cargo.toml -- --target-pane $P_SHELL" C-m

# Initialize the hook in the right shell automatically
tmux send-keys -t "$P_SHELL" "eval \"\$(cargo run --manifest-path=/home/chris/03.projects/marlino/Cargo.toml -- --init-zsh)\"" C-m
tmux send-keys -t "$P_SHELL" "clear" C-m

# Hide the dividing border for this specific window
tmux set-window-option -t "$W_ID" pane-border-style "fg=#282726,bg=#282726"
tmux set-window-option -t "$W_ID" pane-active-border-style "fg=#282726,bg=#282726"

tmux select-pane -t "$P_SHELL"
