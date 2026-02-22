use std::cmp::min;
use std::io;
use std::sync::Arc;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::{prelude::*, Terminal};
use shaku::{Component, Interface};
use sublime_fuzzy::best_match;
use unicode_width::UnicodeWidthChar;

use crate::commands::tmux::session_utils::SessionUtilsService;
use crate::domain::tmux_workspaces::aggregates::tmux::description::session::SessionDescription;
use crate::domain::tmux_workspaces::repositories::tmux::client_repository::{
    SwitchClientTarget, TmuxClientRepository,
};
use crate::domain::tmux_workspaces::repositories::tmux::description_repository::SessionDescriptionRepository;
use crate::domain::tmux_workspaces::repositories::tmux::session_repository::TmuxSessionRepository;
use crate::utils::with_terminal;

pub trait TmuxSwitchCommandInterface: Interface {
    fn execute(&self);
}

#[derive(Component)]
#[shaku(interface = TmuxSwitchCommandInterface)]
pub struct TmuxSwitchCommand {
    #[shaku(inject)]
    session_description_repository: Arc<dyn SessionDescriptionRepository>,
    #[shaku(inject)]
    session_repository: Arc<dyn TmuxSessionRepository>,
    #[shaku(inject)]
    client_repository: Arc<dyn TmuxClientRepository>,
    #[shaku(inject)]
    session_utils: Arc<dyn SessionUtilsService>,
}

impl TmuxSwitchCommandInterface for TmuxSwitchCommand {
    fn execute(&self) {
        let descriptions = self
            .session_description_repository
            .get_session_descriptions();

        let res = fuzzy_pick(FuzzySearchArgs {
            items: &descriptions,
            search_text_fun: select_name,
        })
        .expect("Hey");

        if let Some(selected_session) = res {
            println!("You selected {}!", selected_session.name);
            let session = match &selected_session.session {
                Some(se) => se,
                None => &self.session_repository.new_session(selected_session),
            };

            self.client_repository
                .switch_client(None, SwitchClientTarget::Session(session));

            // Create worktree sessions if this is a workspace session
            use crate::domain::tmux_workspaces::aggregates::tmux::description::session::SessionKind;
            if let SessionKind::Workspace(workspace) = &selected_session.kind {
                self.session_utils.create_worktree_sessions(workspace);
            }
        } else {
            println!("You didn't make a selection :/");
        }
    }
}

fn select_name(desc: &SessionDescription) -> &str {
    &desc.name
}

struct FuzzySearchArgs<'a, T, TFn>
where
    TFn: Fn(&'a T) -> &str,
{
    items: &'a [T],
    search_text_fun: TFn,
}

fn fuzzy_pick_base<'a, T, TFn>(
    args: FuzzySearchArgs<'a, T, TFn>,
    terminal: &'_ mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<Option<&'a T>>
where
    TFn: Fn(&T) -> &str,
{
    let search_text_fun = args.search_text_fun;
    // Prepare owned refs to keep lifetimes simple
    let owned: Vec<String> = args
        .items
        .iter()
        .map(|s| search_text_fun(s).to_string())
        .collect();

    let mut query = String::new();
    let mut selected_idx: usize = 0;

    // Cached filtered view: (score desc, index into owned)
    let mut filtered: Vec<(i64, usize)> = rebuild_filtered(&owned, &query);

    let res = loop {
        terminal.draw(|f| {
            let size = f.area();

            // Layout: input, list, help
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(3),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(size);

            // Input area
            let input = Paragraph::new(Line::from(vec![
                Span::styled("Query: ", Style::default().fg(Color::Cyan)),
                Span::raw(&query),
            ]))
            .block(Block::default().borders(Borders::ALL).title("Fuzzy Picker"));
            f.render_widget(input, chunks[0]);

            // Build list items from filtered
            let list_items: Vec<ListItem> = if filtered.is_empty() && !query.is_empty() {
                vec![ListItem::new("No matches")]
            } else {
                filtered
                    .iter()
                    .map(|&(_score, idx)| {
                        let text = &owned[idx];
                        ListItem::new(text.clone())
                    })
                    .collect()
            };

            let list = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title("Matches"))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

            // Clamp selected index to filtered length
            if filtered.is_empty() {
                selected_idx = 0;
            } else {
                selected_idx = min(selected_idx, filtered.len().saturating_sub(1));
            }

            // Render stateful list selection
            let mut state = ratatui::widgets::ListState::default();
            if !filtered.is_empty() || query.is_empty() {
                state.select(Some(selected_idx));
            }
            f.render_stateful_widget(list, chunks[1], &mut state);

            // Help/footer
            let help = Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(" confirm  "),
                Span::styled("Esc/q/Ctrl-C", Style::default().fg(Color::Red)),
                Span::raw(" cancel  "),
                Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
                Span::raw(" navigate  "),
                Span::styled("Type", Style::default().fg(Color::Magenta)),
                Span::raw(" to filter"),
            ]);
            let help_p = Paragraph::new(help);
            f.render_widget(help_p, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => {
                    // Navigation
                    match (code, modifiers) {
                        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                            if !filtered.is_empty() {
                                if selected_idx == 0 {
                                    selected_idx = filtered.len() - 1;
                                } else {
                                    selected_idx -= 1;
                                }
                            }
                            continue;
                        }
                        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                            if !filtered.is_empty() {
                                selected_idx = (selected_idx + 1) % filtered.len();
                            }
                            continue;
                        }
                        _ => {}
                    }

                    match code {
                        KeyCode::Enter => {
                            let choice = if filtered.is_empty() {
                                None
                            } else {
                                let idx = filtered[selected_idx].1;
                                Some(&args.items[idx])
                            };
                            break choice;
                        }
                        KeyCode::Esc | KeyCode::Char('q') => break None,
                        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                            break None
                        }
                        KeyCode::Backspace => {
                            if !query.is_empty() {
                                query.pop();
                                filtered = rebuild_filtered(&owned, &query);
                                selected_idx = 0;
                            }
                        }
                        KeyCode::Char(ch) => {
                            if !modifiers.contains(KeyModifiers::CONTROL)
                                && !modifiers.contains(KeyModifiers::ALT)
                            {
                                if ch.width().unwrap() > 0 {
                                    query.push(ch);
                                    filtered = rebuild_filtered(&owned, &query);
                                    selected_idx = 0;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::Paste(s) => {
                    if !s.is_empty() {
                        query.push_str(&s);
                        filtered = rebuild_filtered(&owned, &query);
                        selected_idx = 0;
                    }
                }
                Event::Resize(_, _) => {
                    // Redraw on next loop
                }
                _ => {}
            }
        }
    };

    Ok(res)
}

/// A simple fuzzy-search picker:
/// - Arrow Up/Down or Ctrl-K/Ctrl-J to move selection
/// - Enter to confirm
/// - Esc, q, or Ctrl-C to cancel
/// - Typing filters; Backspace deletes
///
/// Returns the chosen item, or None if canceled.
fn fuzzy_pick<T, TFn>(args: FuzzySearchArgs<'_, T, TFn>) -> io::Result<Option<&'_ T>>
where
    TFn: Fn(&T) -> &str,
{
    with_terminal::with_terminal(|terminal| fuzzy_pick_base(args, terminal))
}

fn rebuild_filtered(items: &[impl AsRef<str>], query: &str) -> Vec<(i64, usize)> {
    if query.trim().is_empty() {
        // Show all with neutral scores, preserve original order
        return items
            .iter()
            .enumerate()
            .map(|(i, _)| (0, i))
            .collect::<Vec<_>>();
    }

    let mut scored: Vec<(i64, usize)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            // best_match returns Option<Match>; its score is higher for better match
            best_match(query, s.as_ref()).map(|m| (m.score() as i64, i))
        })
        .collect();

    // Sort by descending score, then by index for stability
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    scored
}
