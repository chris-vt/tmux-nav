#!/usr/bin/env bash
W_ID=$(tmux new-window -P -F "#{window_id}")
P_SHELL=$(tmux display-message -t "$W_ID.0" -p "#{pane_id}")
P_NAV=$(tmux split-window -t "$W_ID" -h -l 15% -b -P -F "#{pane_id}" "/home/chris/03.projects/tmux-nav/result/bin/tmux-nav --target-pane '$P_SHELL'")

# Hide the dividing border for this specific window
tmux set-window-option -t "$W_ID" pane-border-style "fg=#282726,bg=#282726"
tmux set-window-option -t "$W_ID" pane-active-border-style "fg=#282726,bg=#282726"

tmux select-pane -t "$P_SHELL"
