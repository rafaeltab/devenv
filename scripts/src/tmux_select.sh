__tmux_fzf_get_session__() {
    session=$(__tmux_list_sessions__ |
        fzf --with-nth=2.. --print0 --preview="echo {} | cut -d' ' -f1 | xargs -I{} tmux_preview.sh {}" | cut -d' ' -f1)
    tmux switch-client -t $session
}

__tmux_list_sessions__() {
    tmux list-sessions -F "#{session_id} #{session_name}" 2>/dev/null
}


#__tmux_list_sessions__
__tmux_fzf_get_session__
