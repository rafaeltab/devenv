session_id="$1"

# Get the active window for the session
active_window=$(tmux list-windows -t "$session_id" -F "#{window_id} #{window_active}" | awk '$2=="1" {print $1}')

# Get the active pane for that window
active_pane=$(tmux list-panes -t "$active_window" -F "#{pane_id} #{pane_active}" | awk '$2=="1" {print $1}')

# echo "Session: $session_id"
# echo "Active Window: $active_window"
# echo "Active Pane: $active_pane"

tmux capture-pane -t $active_pane -p -e
