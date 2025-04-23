#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TmuxFormatVariable {
    /// Index of active window in session
    ActiveWindowIndex,
    /// 1 if pane is in alternate screen
    AlternateOn,
    /// Saved cursor X in alternate screen
    AlternateSavedX,
    /// Saved cursor Y in alternate screen
    AlternateSavedY,
    /// Time buffer created
    BufferCreated,
    /// Name of buffer
    BufferName,
    /// Sample of start of buffer
    BufferSample,
    /// Size of the specified buffer in bytes
    BufferSize,
    /// Time client last had activity
    ClientActivity,
    /// Height of each client cell in pixels
    ClientCellHeight,
    /// Width of each client cell in pixels
    ClientCellWidth,
    /// 1 if client is in control mode
    ClientControlMode,
    /// Time client created
    ClientCreated,
    /// Bytes discarded when client behind
    ClientDiscarded,
    /// List of client flags
    ClientFlags,
    /// Height of client
    ClientHeight,
    /// Current key table
    ClientKeyTable,
    /// Name of the client's last session
    ClientLastSession,
    /// Name of client
    ClientName,
    /// PID of client process
    ClientPid,
    /// 1 if prefix key has been pressed
    ClientPrefix,
    /// 1 if client is read-only
    ClientReadonly,
    /// Name of the client's session
    ClientSession,
    /// Terminal features of client, if any
    ClientTermfeatures,
    /// Terminal name of client
    ClientTermname,
    /// Terminal type of client, if available
    ClientTermtype,
    /// Pseudo terminal of client
    ClientTty,
    /// UID of client process
    ClientUid,
    /// User of client process
    ClientUser,
    /// 1 if client supports UTF-8
    ClientUtf8,
    /// Width of client
    ClientWidth,
    /// Bytes written to client
    ClientWritten,
    /// Name of command in use, if any
    Command,
    /// Command alias if listing commands
    CommandListAlias,
    /// Command name if listing commands
    CommandListName,
    /// Command usage if listing commands
    CommandListUsage,
    /// List of configuration files loaded
    ConfigFiles,
    /// Line the cursor is on in copy mode
    CopyCursorLine,
    /// Word under cursor in copy mode
    CopyCursorWord,
    /// Cursor X position in copy mode
    CopyCursorX,
    /// Cursor Y position in copy mode
    CopyCursorY,
    /// Current configuration file
    CurrentFile,
    /// Character at cursor in pane
    CursorCharacter,
    /// Pane cursor flag
    CursorFlag,
    /// Cursor X position in pane
    CursorX,
    /// Cursor Y position in pane
    CursorY,
    /// Number of bytes in window history
    HistoryBytes,
    /// Maximum window history lines
    HistoryLimit,
    /// Size of history in lines
    HistorySize,
    /// Name of running hook, if any
    Hook,
    /// Name of client where hook was run, if any
    HookClient,
    /// ID of pane where hook was run, if any
    HookPane,
    /// ID of session where hook was run, if any
    HookSession,
    /// Name of session where hook was run, if any
    HookSessionName,
    /// ID of window where hook was run, if any
    HookWindow,
    /// Name of window where hook was run, if any
    HookWindowName,
    /// Hostname of local host
    Host,
    /// Hostname of local host (no domain name)
    HostShort,
    /// Pane insert flag
    InsertFlag,
    /// Pane keypad cursor flag
    KeypadCursorFlag,
    /// Pane keypad flag
    KeypadFlag,
    /// Index of last window in session
    LastWindowIndex,
    /// Line number in the list
    Line,
    /// Pane mouse all flag
    MouseAllFlag,
    /// Pane mouse any flag
    MouseAnyFlag,
    /// Pane mouse button flag
    MouseButtonFlag,
    /// Hyperlink under mouse, if any
    MouseHyperlink,
    /// Line under mouse, if any
    MouseLine,
    /// Pane mouse SGR flag
    MouseSgrFlag,
    /// Pane mouse standard flag
    MouseStandardFlag,
    /// Status line on which mouse event took place
    MouseStatusLine,
    /// Range type or argument of mouse event on status line
    MouseStatusRange,
    /// Pane mouse UTF-8 flag
    MouseUtf8Flag,
    /// Word under mouse, if any
    MouseWord,
    /// Mouse X position, if any
    MouseX,
    /// Mouse Y position, if any
    MouseY,
    /// Unique session ID for next new session
    NextSessionId,
    /// Pane origin flag
    OriginFlag,
    /// 1 if active pane
    PaneActive,
    /// 1 if pane is at the bottom of window
    PaneAtBottom,
    /// 1 if pane is at the left of window
    PaneAtLeft,
    /// 1 if pane is at the right of window
    PaneAtRight,
    /// 1 if pane is at the top of window
    PaneAtTop,
    /// Pane background colour
    PaneBg,
    /// Bottom of pane
    PaneBottom,
    /// Current command if available
    PaneCurrentCommand,
    /// Current path if available
    PaneCurrentPath,
    /// 1 if pane is dead
    PaneDead,
    /// Exit signal of process in dead pane
    PaneDeadSignal,
    /// Exit status of process in dead pane
    PaneDeadStatus,
    /// Exit time of process in dead pane
    PaneDeadTime,
    /// Pane foreground colour
    PaneFg,
    /// 1 if format is for a pane
    PaneFormat,
    /// Height of pane
    PaneHeight,
    /// Unique pane ID
    PaneId,
    /// 1 if pane is in a mode
    PaneInMode,
    /// Index of pane
    PaneIndex,
    /// 1 if input to pane is disabled
    PaneInputOff,
    /// 1 if last pane
    PaneLast,
    /// Left of pane
    PaneLeft,
    /// 1 if this is the marked pane
    PaneMarked,
    /// 1 if a marked pane is set
    PaneMarkedSet,
    /// Name of pane mode, if any
    PaneMode,
    /// Path of pane (can be set by application)
    PanePath,
    /// PID of first process in pane
    PanePid,
    /// 1 if pane is being piped
    PanePipe,
    /// Right of pane
    PaneRight,
    /// Last search string in copy mode
    PaneSearchString,
    /// Command pane started with
    PaneStartCommand,
    /// Path pane started with
    PaneStartPath,
    /// 1 if pane is synchronized
    PaneSynchronized,
    /// Pane tab positions
    PaneTabs,
    /// Title of pane (can be set by application)
    PaneTitle,
    /// Top of pane
    PaneTop,
    /// Pseudo terminal of pane
    PaneTty,
    /// 1 if there were changes in pane while in mode
    PaneUnseenChanges,
    /// Width of pane
    PaneWidth,
    /// Server PID
    Pid,
    /// 1 if rectangle selection is activated
    RectangleToggle,
    /// Scroll position in copy mode
    ScrollPosition,
    /// Bottom of scroll region in pane
    ScrollRegionLower,
    /// Top of scroll region in pane
    ScrollRegionUpper,
    /// Search match if any
    SearchMatch,
    /// 1 if search started in copy mode
    SearchPresent,
    /// 1 if selection started and changes with the cursor in copy mode
    SelectionActive,
    /// X position of the end of the selection
    SelectionEndX,
    /// Y position of the end of the selection
    SelectionEndY,
    /// 1 if selection started in copy mode
    SelectionPresent,
    /// X position of the start of the selection
    SelectionStartX,
    /// Y position of the start of the selection
    SelectionStartY,
    /// Number of sessions
    ServerSessions,
    /// Time of session last activity
    SessionActivity,
    /// List of window indexes with alerts
    SessionAlerts,
    /// Number of clients session is attached to
    SessionAttached,
    /// List of clients session is attached to
    SessionAttachedList,
    /// Time session created
    SessionCreated,
    /// 1 if format is for a session
    SessionFormat,
    /// Name of session group
    SessionGroup,
    /// Number of clients sessions in group are attached to
    SessionGroupAttached,
    /// List of clients sessions in group are attached to
    SessionGroupAttachedList,
    /// List of sessions in group
    SessionGroupList,
    /// 1 if multiple clients attached to sessions in group
    SessionGroupManyAttached,
    /// Size of session group
    SessionGroupSize,
    /// 1 if session in a group
    SessionGrouped,
    /// Unique session ID
    SessionId,
    /// Time session last attached
    SessionLastAttached,
    /// 1 if multiple clients attached
    SessionManyAttached,
    /// 1 if this session contains the marked pane
    SessionMarked,
    /// Name of session
    SessionName,
    /// Working directory of session
    SessionPath,
    /// Window indexes in most recent order
    SessionStack,
    /// Number of windows in session
    SessionWindows,
    /// Server socket path
    SocketPath,
    /// Server start time
    StartTime,
    /// Server UID
    Uid,
    /// Server user
    User,
    /// Server version
    Version,
    /// 1 if window active
    WindowActive,
    /// Number of clients viewing this window
    WindowActiveClients,
    /// List of clients viewing this window
    WindowActiveClientsList,
    /// Number of sessions on which this window is active
    WindowActiveSessions,
    /// List of sessions on which this window is active
    WindowActiveSessionsList,
    /// Time of window last activity
    WindowActivity,
    /// 1 if window has activity
    WindowActivityFlag,
    /// 1 if window has bell
    WindowBellFlag,
    /// 1 if window is larger than client
    WindowBigger,
    /// Height of each cell in pixels
    WindowCellHeight,
    /// Width of each cell in pixels
    WindowCellWidth,
    /// 1 if window has the highest index
    WindowEndFlag,
    /// Window flags with # escaped as ##
    WindowFlags,
    /// 1 if format is for a window
    WindowFormat,
    /// Height of window
    WindowHeight,
    /// Unique window ID
    WindowId,
    /// Index of window
    WindowIndex,
    /// 1 if window is the last used
    WindowLastFlag,
    /// Window layout description, ignoring zoomed window panes
    WindowLayout,
    /// 1 if window is linked across sessions
    WindowLinked,
    /// Number of sessions this window is linked to
    WindowLinkedSessions,
    /// List of sessions this window is linked to
    WindowLinkedSessionsList,
    /// 1 if window contains the marked pane
    WindowMarkedFlag,
    /// Name of window
    WindowName,
    /// X offset into window if larger than client
    WindowOffsetX,
    /// Y offset into window if larger than client
    WindowOffsetY,
    /// Number of panes in window
    WindowPanes,
    /// Window flags with nothing escaped
    WindowRawFlags,
    /// 1 if window has silence alert
    WindowSilenceFlag,
    /// Index in session most recent stack
    WindowStackIndex,
    /// 1 if window has the lowest index
    WindowStartFlag,
    /// Window layout description, respecting zoomed window panes
    WindowVisibleLayout,
    /// Width of window
    WindowWidth,
    /// 1 if window is zoomed
    WindowZoomedFlag,
    /// Pane wrap flag
    WrapFlag,
}
impl From<&str> for TmuxFormatVariable {
    fn from(s: &str) -> Self {
        match s {
            "active_window_index" => TmuxFormatVariable::ActiveWindowIndex,
            "alternate_on" => TmuxFormatVariable::AlternateOn,
            "alternate_saved_x" => TmuxFormatVariable::AlternateSavedX,
            "alternate_saved_y" => TmuxFormatVariable::AlternateSavedY,
            "buffer_created" => TmuxFormatVariable::BufferCreated,
            "buffer_name" => TmuxFormatVariable::BufferName,
            "buffer_sample" => TmuxFormatVariable::BufferSample,
            "buffer_size" => TmuxFormatVariable::BufferSize,
            "client_activity" => TmuxFormatVariable::ClientActivity,
            "client_cell_height" => TmuxFormatVariable::ClientCellHeight,
            "client_cell_width" => TmuxFormatVariable::ClientCellWidth,
            "client_control_mode" => TmuxFormatVariable::ClientControlMode,
            "client_created" => TmuxFormatVariable::ClientCreated,
            "client_discarded" => TmuxFormatVariable::ClientDiscarded,
            "client_flags" => TmuxFormatVariable::ClientFlags,
            "client_height" => TmuxFormatVariable::ClientHeight,
            "client_key_table" => TmuxFormatVariable::ClientKeyTable,
            "client_last_session" => TmuxFormatVariable::ClientLastSession,
            "client_name" => TmuxFormatVariable::ClientName,
            "client_pid" => TmuxFormatVariable::ClientPid,
            "client_prefix" => TmuxFormatVariable::ClientPrefix,
            "client_readonly" => TmuxFormatVariable::ClientReadonly,
            "client_session" => TmuxFormatVariable::ClientSession,
            "client_termfeatures" => TmuxFormatVariable::ClientTermfeatures,
            "client_termname" => TmuxFormatVariable::ClientTermname,
            "client_termtype" => TmuxFormatVariable::ClientTermtype,
            "client_tty" => TmuxFormatVariable::ClientTty,
            "client_uid" => TmuxFormatVariable::ClientUid,
            "client_user" => TmuxFormatVariable::ClientUser,
            "client_utf8" => TmuxFormatVariable::ClientUtf8,
            "client_width" => TmuxFormatVariable::ClientWidth,
            "client_written" => TmuxFormatVariable::ClientWritten,
            "command" => TmuxFormatVariable::Command,
            "command_list_alias" => TmuxFormatVariable::CommandListAlias,
            "command_list_name" => TmuxFormatVariable::CommandListName,
            "command_list_usage" => TmuxFormatVariable::CommandListUsage,
            "config_files" => TmuxFormatVariable::ConfigFiles,
            "copy_cursor_line" => TmuxFormatVariable::CopyCursorLine,
            "copy_cursor_word" => TmuxFormatVariable::CopyCursorWord,
            "copy_cursor_x" => TmuxFormatVariable::CopyCursorX,
            "copy_cursor_y" => TmuxFormatVariable::CopyCursorY,
            "current_file" => TmuxFormatVariable::CurrentFile,
            "cursor_character" => TmuxFormatVariable::CursorCharacter,
            "cursor_flag" => TmuxFormatVariable::CursorFlag,
            "cursor_x" => TmuxFormatVariable::CursorX,
            "cursor_y" => TmuxFormatVariable::CursorY,
            "history_bytes" => TmuxFormatVariable::HistoryBytes,
            "history_limit" => TmuxFormatVariable::HistoryLimit,
            "history_size" => TmuxFormatVariable::HistorySize,
            "hook" => TmuxFormatVariable::Hook,
            "hook_client" => TmuxFormatVariable::HookClient,
            "hook_pane" => TmuxFormatVariable::HookPane,
            "hook_session" => TmuxFormatVariable::HookSession,
            "hook_session_name" => TmuxFormatVariable::HookSessionName,
            "hook_window" => TmuxFormatVariable::HookWindow,
            "hook_window_name" => TmuxFormatVariable::HookWindowName,
            "host" => TmuxFormatVariable::Host,
            "host_short" => TmuxFormatVariable::HostShort,
            "insert_flag" => TmuxFormatVariable::InsertFlag,
            "keypad_cursor_flag" => TmuxFormatVariable::KeypadCursorFlag,
            "keypad_flag" => TmuxFormatVariable::KeypadFlag,
            "last_window_index" => TmuxFormatVariable::LastWindowIndex,
            "line" => TmuxFormatVariable::Line,
            "mouse_all_flag" => TmuxFormatVariable::MouseAllFlag,
            "mouse_any_flag" => TmuxFormatVariable::MouseAnyFlag,
            "mouse_button_flag" => TmuxFormatVariable::MouseButtonFlag,
            "mouse_hyperlink" => TmuxFormatVariable::MouseHyperlink,
            "mouse_line" => TmuxFormatVariable::MouseLine,
            "mouse_sgr_flag" => TmuxFormatVariable::MouseSgrFlag,
            "mouse_standard_flag" => TmuxFormatVariable::MouseStandardFlag,
            "mouse_status_line" => TmuxFormatVariable::MouseStatusLine,
            "mouse_status_range" => TmuxFormatVariable::MouseStatusRange,
            "mouse_utf8_flag" => TmuxFormatVariable::MouseUtf8Flag,
            "mouse_word" => TmuxFormatVariable::MouseWord,
            "mouse_x" => TmuxFormatVariable::MouseX,
            "mouse_y" => TmuxFormatVariable::MouseY,
            "next_session_id" => TmuxFormatVariable::NextSessionId,
            "origin_flag" => TmuxFormatVariable::OriginFlag,
            "pane_active" => TmuxFormatVariable::PaneActive,
            "pane_at_bottom" => TmuxFormatVariable::PaneAtBottom,
            "pane_at_left" => TmuxFormatVariable::PaneAtLeft,
            "pane_at_right" => TmuxFormatVariable::PaneAtRight,
            "pane_at_top" => TmuxFormatVariable::PaneAtTop,
            "pane_bg" => TmuxFormatVariable::PaneBg,
            "pane_bottom" => TmuxFormatVariable::PaneBottom,
            "pane_current_command" => TmuxFormatVariable::PaneCurrentCommand,
            "pane_current_path" => TmuxFormatVariable::PaneCurrentPath,
            "pane_dead" => TmuxFormatVariable::PaneDead,
            "pane_dead_signal" => TmuxFormatVariable::PaneDeadSignal,
            "pane_dead_status" => TmuxFormatVariable::PaneDeadStatus,
            "pane_dead_time" => TmuxFormatVariable::PaneDeadTime,
            "pane_fg" => TmuxFormatVariable::PaneFg,
            "pane_format" => TmuxFormatVariable::PaneFormat,
            "pane_height" => TmuxFormatVariable::PaneHeight,
            "pane_id" => TmuxFormatVariable::PaneId,
            "pane_in_mode" => TmuxFormatVariable::PaneInMode,
            "pane_index" => TmuxFormatVariable::PaneIndex,
            "pane_input_off" => TmuxFormatVariable::PaneInputOff,
            "pane_last" => TmuxFormatVariable::PaneLast,
            "pane_left" => TmuxFormatVariable::PaneLeft,
            "pane_marked" => TmuxFormatVariable::PaneMarked,
            "pane_marked_set" => TmuxFormatVariable::PaneMarkedSet,
            "pane_mode" => TmuxFormatVariable::PaneMode,
            "pane_path" => TmuxFormatVariable::PanePath,
            "pane_pid" => TmuxFormatVariable::PanePid,
            "pane_pipe" => TmuxFormatVariable::PanePipe,
            "pane_right" => TmuxFormatVariable::PaneRight,
            "pane_search_string" => TmuxFormatVariable::PaneSearchString,
            "pane_start_command" => TmuxFormatVariable::PaneStartCommand,
            "pane_start_path" => TmuxFormatVariable::PaneStartPath,
            "pane_synchronized" => TmuxFormatVariable::PaneSynchronized,
            "pane_tabs" => TmuxFormatVariable::PaneTabs,
            "pane_title" => TmuxFormatVariable::PaneTitle,
            "pane_top" => TmuxFormatVariable::PaneTop,
            "pane_tty" => TmuxFormatVariable::PaneTty,
            "pane_unseen_changes" => TmuxFormatVariable::PaneUnseenChanges,
            "pane_width" => TmuxFormatVariable::PaneWidth,
            "pid" => TmuxFormatVariable::Pid,
            "rectangle_toggle" => TmuxFormatVariable::RectangleToggle,
            "scroll_position" => TmuxFormatVariable::ScrollPosition,
            "scroll_region_lower" => TmuxFormatVariable::ScrollRegionLower,
            "scroll_region_upper" => TmuxFormatVariable::ScrollRegionUpper,
            "search_match" => TmuxFormatVariable::SearchMatch,
            "search_present" => TmuxFormatVariable::SearchPresent,
            "selection_active" => TmuxFormatVariable::SelectionActive,
            "selection_end_x" => TmuxFormatVariable::SelectionEndX,
            "selection_end_y" => TmuxFormatVariable::SelectionEndY,
            "selection_present" => TmuxFormatVariable::SelectionPresent,
            "selection_start_x" => TmuxFormatVariable::SelectionStartX,
            "selection_start_y" => TmuxFormatVariable::SelectionStartY,
            "server_sessions" => TmuxFormatVariable::ServerSessions,
            "session_activity" => TmuxFormatVariable::SessionActivity,
            "session_alerts" => TmuxFormatVariable::SessionAlerts,
            "session_attached" => TmuxFormatVariable::SessionAttached,
            "session_attached_list" => TmuxFormatVariable::SessionAttachedList,
            "session_created" => TmuxFormatVariable::SessionCreated,
            "session_format" => TmuxFormatVariable::SessionFormat,
            "session_group" => TmuxFormatVariable::SessionGroup,
            "session_group_attached" => TmuxFormatVariable::SessionGroupAttached,
            "session_group_attached_list" => TmuxFormatVariable::SessionGroupAttachedList,
            "session_group_list" => TmuxFormatVariable::SessionGroupList,
            "session_group_many_attached" => TmuxFormatVariable::SessionGroupManyAttached,
            "session_group_size" => TmuxFormatVariable::SessionGroupSize,
            "session_grouped" => TmuxFormatVariable::SessionGrouped,
            "session_id" => TmuxFormatVariable::SessionId,
            "session_last_attached" => TmuxFormatVariable::SessionLastAttached,
            "session_many_attached" => TmuxFormatVariable::SessionManyAttached,
            "session_marked" => TmuxFormatVariable::SessionMarked,
            "session_name" => TmuxFormatVariable::SessionName,
            "session_path" => TmuxFormatVariable::SessionPath,
            "session_stack" => TmuxFormatVariable::SessionStack,
            "session_windows" => TmuxFormatVariable::SessionWindows,
            "socket_path" => TmuxFormatVariable::SocketPath,
            "start_time" => TmuxFormatVariable::StartTime,
            "uid" => TmuxFormatVariable::Uid,
            "user" => TmuxFormatVariable::User,
            "version" => TmuxFormatVariable::Version,
            "window_active" => TmuxFormatVariable::WindowActive,
            "window_active_clients" => TmuxFormatVariable::WindowActiveClients,
            "window_active_clients_list" => TmuxFormatVariable::WindowActiveClientsList,
            "window_active_sessions" => TmuxFormatVariable::WindowActiveSessions,
            "window_active_sessions_list" => TmuxFormatVariable::WindowActiveSessionsList,
            "window_activity" => TmuxFormatVariable::WindowActivity,
            "window_activity_flag" => TmuxFormatVariable::WindowActivityFlag,
            "window_bell_flag" => TmuxFormatVariable::WindowBellFlag,
            "window_bigger" => TmuxFormatVariable::WindowBigger,
            "window_cell_height" => TmuxFormatVariable::WindowCellHeight,
            "window_cell_width" => TmuxFormatVariable::WindowCellWidth,
            "window_end_flag" => TmuxFormatVariable::WindowEndFlag,
            "window_flags" => TmuxFormatVariable::WindowFlags,
            "window_format" => TmuxFormatVariable::WindowFormat,
            "window_height" => TmuxFormatVariable::WindowHeight,
            "window_id" => TmuxFormatVariable::WindowId,
            "window_index" => TmuxFormatVariable::WindowIndex,
            "window_last_flag" => TmuxFormatVariable::WindowLastFlag,
            "window_layout" => TmuxFormatVariable::WindowLayout,
            "window_linked" => TmuxFormatVariable::WindowLinked,
            "window_linked_sessions" => TmuxFormatVariable::WindowLinkedSessions,
            "window_linked_sessions_list" => TmuxFormatVariable::WindowLinkedSessionsList,
            "window_marked_flag" => TmuxFormatVariable::WindowMarkedFlag,
            "window_name" => TmuxFormatVariable::WindowName,
            "window_offset_x" => TmuxFormatVariable::WindowOffsetX,
            "window_offset_y" => TmuxFormatVariable::WindowOffsetY,
            "window_panes" => TmuxFormatVariable::WindowPanes,
            "window_raw_flags" => TmuxFormatVariable::WindowRawFlags,
            "window_silence_flag" => TmuxFormatVariable::WindowSilenceFlag,
            "window_stack_index" => TmuxFormatVariable::WindowStackIndex,
            "window_start_flag" => TmuxFormatVariable::WindowStartFlag,
            "window_visible_layout" => TmuxFormatVariable::WindowVisibleLayout,
            "window_width" => TmuxFormatVariable::WindowWidth,
            "window_zoomed_flag" => TmuxFormatVariable::WindowZoomedFlag,
            "wrap_flag" => TmuxFormatVariable::WrapFlag,
            _ => panic!("Unknown TmuxFormatVariable: {}", s),
        }
    }
}

impl From<TmuxFormatVariable> for &'static str {
    fn from(val: TmuxFormatVariable) -> Self {
        match val {
            TmuxFormatVariable::ActiveWindowIndex => "active_window_index",
            TmuxFormatVariable::AlternateOn => "alternate_on",
            TmuxFormatVariable::AlternateSavedX => "alternate_saved_x",
            TmuxFormatVariable::AlternateSavedY => "alternate_saved_y",
            TmuxFormatVariable::BufferCreated => "buffer_created",
            TmuxFormatVariable::BufferName => "buffer_name",
            TmuxFormatVariable::BufferSample => "buffer_sample",
            TmuxFormatVariable::BufferSize => "buffer_size",
            TmuxFormatVariable::ClientActivity => "client_activity",
            TmuxFormatVariable::ClientCellHeight => "client_cell_height",
            TmuxFormatVariable::ClientCellWidth => "client_cell_width",
            TmuxFormatVariable::ClientControlMode => "client_control_mode",
            TmuxFormatVariable::ClientCreated => "client_created",
            TmuxFormatVariable::ClientDiscarded => "client_discarded",
            TmuxFormatVariable::ClientFlags => "client_flags",
            TmuxFormatVariable::ClientHeight => "client_height",
            TmuxFormatVariable::ClientKeyTable => "client_key_table",
            TmuxFormatVariable::ClientLastSession => "client_last_session",
            TmuxFormatVariable::ClientName => "client_name",
            TmuxFormatVariable::ClientPid => "client_pid",
            TmuxFormatVariable::ClientPrefix => "client_prefix",
            TmuxFormatVariable::ClientReadonly => "client_readonly",
            TmuxFormatVariable::ClientSession => "client_session",
            TmuxFormatVariable::ClientTermfeatures => "client_termfeatures",
            TmuxFormatVariable::ClientTermname => "client_termname",
            TmuxFormatVariable::ClientTermtype => "client_termtype",
            TmuxFormatVariable::ClientTty => "client_tty",
            TmuxFormatVariable::ClientUid => "client_uid",
            TmuxFormatVariable::ClientUser => "client_user",
            TmuxFormatVariable::ClientUtf8 => "client_utf8",
            TmuxFormatVariable::ClientWidth => "client_width",
            TmuxFormatVariable::ClientWritten => "client_written",
            TmuxFormatVariable::Command => "command",
            TmuxFormatVariable::CommandListAlias => "command_list_alias",
            TmuxFormatVariable::CommandListName => "command_list_name",
            TmuxFormatVariable::CommandListUsage => "command_list_usage",
            TmuxFormatVariable::ConfigFiles => "config_files",
            TmuxFormatVariable::CopyCursorLine => "copy_cursor_line",
            TmuxFormatVariable::CopyCursorWord => "copy_cursor_word",
            TmuxFormatVariable::CopyCursorX => "copy_cursor_x",
            TmuxFormatVariable::CopyCursorY => "copy_cursor_y",
            TmuxFormatVariable::CurrentFile => "current_file",
            TmuxFormatVariable::CursorCharacter => "cursor_character",
            TmuxFormatVariable::CursorFlag => "cursor_flag",
            TmuxFormatVariable::CursorX => "cursor_x",
            TmuxFormatVariable::CursorY => "cursor_y",
            TmuxFormatVariable::HistoryBytes => "history_bytes",
            TmuxFormatVariable::HistoryLimit => "history_limit",
            TmuxFormatVariable::HistorySize => "history_size",
            TmuxFormatVariable::Hook => "hook",
            TmuxFormatVariable::HookClient => "hook_client",
            TmuxFormatVariable::HookPane => "hook_pane",
            TmuxFormatVariable::HookSession => "hook_session",
            TmuxFormatVariable::HookSessionName => "hook_session_name",
            TmuxFormatVariable::HookWindow => "hook_window",
            TmuxFormatVariable::HookWindowName => "hook_window_name",
            TmuxFormatVariable::Host => "host",
            TmuxFormatVariable::HostShort => "host_short",
            TmuxFormatVariable::InsertFlag => "insert_flag",
            TmuxFormatVariable::KeypadCursorFlag => "keypad_cursor_flag",
            TmuxFormatVariable::KeypadFlag => "keypad_flag",
            TmuxFormatVariable::LastWindowIndex => "last_window_index",
            TmuxFormatVariable::Line => "line",
            TmuxFormatVariable::MouseAllFlag => "mouse_all_flag",
            TmuxFormatVariable::MouseAnyFlag => "mouse_any_flag",
            TmuxFormatVariable::MouseButtonFlag => "mouse_button_flag",
            TmuxFormatVariable::MouseHyperlink => "mouse_hyperlink",
            TmuxFormatVariable::MouseLine => "mouse_line",
            TmuxFormatVariable::MouseSgrFlag => "mouse_sgr_flag",
            TmuxFormatVariable::MouseStandardFlag => "mouse_standard_flag",
            TmuxFormatVariable::MouseStatusLine => "mouse_status_line",
            TmuxFormatVariable::MouseStatusRange => "mouse_status_range",
            TmuxFormatVariable::MouseUtf8Flag => "mouse_utf8_flag",
            TmuxFormatVariable::MouseWord => "mouse_word",
            TmuxFormatVariable::MouseX => "mouse_x",
            TmuxFormatVariable::MouseY => "mouse_y",
            TmuxFormatVariable::NextSessionId => "next_session_id",
            TmuxFormatVariable::OriginFlag => "origin_flag",
            TmuxFormatVariable::PaneActive => "pane_active",
            TmuxFormatVariable::PaneAtBottom => "pane_at_bottom",
            TmuxFormatVariable::PaneAtLeft => "pane_at_left",
            TmuxFormatVariable::PaneAtRight => "pane_at_right",
            TmuxFormatVariable::PaneAtTop => "pane_at_top",
            TmuxFormatVariable::PaneBg => "pane_bg",
            TmuxFormatVariable::PaneBottom => "pane_bottom",
            TmuxFormatVariable::PaneCurrentCommand => "pane_current_command",
            TmuxFormatVariable::PaneCurrentPath => "pane_current_path",
            TmuxFormatVariable::PaneDead => "pane_dead",
            TmuxFormatVariable::PaneDeadSignal => "pane_dead_signal",
            TmuxFormatVariable::PaneDeadStatus => "pane_dead_status",
            TmuxFormatVariable::PaneDeadTime => "pane_dead_time",
            TmuxFormatVariable::PaneFg => "pane_fg",
            TmuxFormatVariable::PaneFormat => "pane_format",
            TmuxFormatVariable::PaneHeight => "pane_height",
            TmuxFormatVariable::PaneId => "pane_id",
            TmuxFormatVariable::PaneInMode => "pane_in_mode",
            TmuxFormatVariable::PaneIndex => "pane_index",
            TmuxFormatVariable::PaneInputOff => "pane_input_off",
            TmuxFormatVariable::PaneLast => "pane_last",
            TmuxFormatVariable::PaneLeft => "pane_left",
            TmuxFormatVariable::PaneMarked => "pane_marked",
            TmuxFormatVariable::PaneMarkedSet => "pane_marked_set",
            TmuxFormatVariable::PaneMode => "pane_mode",
            TmuxFormatVariable::PanePath => "pane_path",
            TmuxFormatVariable::PanePid => "pane_pid",
            TmuxFormatVariable::PanePipe => "pane_pipe",
            TmuxFormatVariable::PaneRight => "pane_right",
            TmuxFormatVariable::PaneSearchString => "pane_search_string",
            TmuxFormatVariable::PaneStartCommand => "pane_start_command",
            TmuxFormatVariable::PaneStartPath => "pane_start_path",
            TmuxFormatVariable::PaneSynchronized => "pane_synchronized",
            TmuxFormatVariable::PaneTabs => "pane_tabs",
            TmuxFormatVariable::PaneTitle => "pane_title",
            TmuxFormatVariable::PaneTop => "pane_top",
            TmuxFormatVariable::PaneTty => "pane_tty",
            TmuxFormatVariable::PaneUnseenChanges => "pane_unseen_changes",
            TmuxFormatVariable::PaneWidth => "pane_width",
            TmuxFormatVariable::Pid => "pid",
            TmuxFormatVariable::RectangleToggle => "rectangle_toggle",
            TmuxFormatVariable::ScrollPosition => "scroll_position",
            TmuxFormatVariable::ScrollRegionLower => "scroll_region_lower",
            TmuxFormatVariable::ScrollRegionUpper => "scroll_region_upper",
            TmuxFormatVariable::SearchMatch => "search_match",
            TmuxFormatVariable::SearchPresent => "search_present",
            TmuxFormatVariable::SelectionActive => "selection_active",
            TmuxFormatVariable::SelectionEndX => "selection_end_x",
            TmuxFormatVariable::SelectionEndY => "selection_end_y",
            TmuxFormatVariable::SelectionPresent => "selection_present",
            TmuxFormatVariable::SelectionStartX => "selection_start_x",
            TmuxFormatVariable::SelectionStartY => "selection_start_y",
            TmuxFormatVariable::ServerSessions => "server_sessions",
            TmuxFormatVariable::SessionActivity => "session_activity",
            TmuxFormatVariable::SessionAlerts => "session_alerts",
            TmuxFormatVariable::SessionAttached => "session_attached",
            TmuxFormatVariable::SessionAttachedList => "session_attached_list",
            TmuxFormatVariable::SessionCreated => "session_created",
            TmuxFormatVariable::SessionFormat => "session_format",
            TmuxFormatVariable::SessionGroup => "session_group",
            TmuxFormatVariable::SessionGroupAttached => "session_group_attached",
            TmuxFormatVariable::SessionGroupAttachedList => "session_group_attached_list",
            TmuxFormatVariable::SessionGroupList => "session_group_list",
            TmuxFormatVariable::SessionGroupManyAttached => "session_group_many_attached",
            TmuxFormatVariable::SessionGroupSize => "session_group_size",
            TmuxFormatVariable::SessionGrouped => "session_grouped",
            TmuxFormatVariable::SessionId => "session_id",
            TmuxFormatVariable::SessionLastAttached => "session_last_attached",
            TmuxFormatVariable::SessionManyAttached => "session_many_attached",
            TmuxFormatVariable::SessionMarked => "session_marked",
            TmuxFormatVariable::SessionName => "session_name",
            TmuxFormatVariable::SessionPath => "session_path",
            TmuxFormatVariable::SessionStack => "session_stack",
            TmuxFormatVariable::SessionWindows => "session_windows",
            TmuxFormatVariable::SocketPath => "socket_path",
            TmuxFormatVariable::StartTime => "start_time",
            TmuxFormatVariable::Uid => "uid",
            TmuxFormatVariable::User => "user",
            TmuxFormatVariable::Version => "version",
            TmuxFormatVariable::WindowActive => "window_active",
            TmuxFormatVariable::WindowActiveClients => "window_active_clients",
            TmuxFormatVariable::WindowActiveClientsList => "window_active_clients_list",
            TmuxFormatVariable::WindowActiveSessions => "window_active_sessions",
            TmuxFormatVariable::WindowActiveSessionsList => "window_active_sessions_list",
            TmuxFormatVariable::WindowActivity => "window_activity",
            TmuxFormatVariable::WindowActivityFlag => "window_activity_flag",
            TmuxFormatVariable::WindowBellFlag => "window_bell_flag",
            TmuxFormatVariable::WindowBigger => "window_bigger",
            TmuxFormatVariable::WindowCellHeight => "window_cell_height",
            TmuxFormatVariable::WindowCellWidth => "window_cell_width",
            TmuxFormatVariable::WindowEndFlag => "window_end_flag",
            TmuxFormatVariable::WindowFlags => "window_flags",
            TmuxFormatVariable::WindowFormat => "window_format",
            TmuxFormatVariable::WindowHeight => "window_height",
            TmuxFormatVariable::WindowId => "window_id",
            TmuxFormatVariable::WindowIndex => "window_index",
            TmuxFormatVariable::WindowLastFlag => "window_last_flag",
            TmuxFormatVariable::WindowLayout => "window_layout",
            TmuxFormatVariable::WindowLinked => "window_linked",
            TmuxFormatVariable::WindowLinkedSessions => "window_linked_sessions",
            TmuxFormatVariable::WindowLinkedSessionsList => "window_linked_sessions_list",
            TmuxFormatVariable::WindowMarkedFlag => "window_marked_flag",
            TmuxFormatVariable::WindowName => "window_name",
            TmuxFormatVariable::WindowOffsetX => "window_offset_x",
            TmuxFormatVariable::WindowOffsetY => "window_offset_y",
            TmuxFormatVariable::WindowPanes => "window_panes",
            TmuxFormatVariable::WindowRawFlags => "window_raw_flags",
            TmuxFormatVariable::WindowSilenceFlag => "window_silence_flag",
            TmuxFormatVariable::WindowStackIndex => "window_stack_index",
            TmuxFormatVariable::WindowStartFlag => "window_start_flag",
            TmuxFormatVariable::WindowVisibleLayout => "window_visible_layout",
            TmuxFormatVariable::WindowWidth => "window_width",
            TmuxFormatVariable::WindowZoomedFlag => "window_zoomed_flag",
            TmuxFormatVariable::WrapFlag => "wrap_flag",
        }
    }
}

pub trait TmuxFormatField {
    fn to_format(self) -> String;
}

impl TmuxFormatField for TmuxFormatVariable {
    fn to_format(self) -> String {
        let string: &str = self.into();
        format!("#{{{}}}", string)
    }
}

impl TmuxFormatVariable {
    pub fn as_string(&self) -> String {
        let string: &str = self.clone().into();
        string.to_string()
    }
}
