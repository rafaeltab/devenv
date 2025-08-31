use std::cmp::min;
use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::{prelude::*, Terminal};
use std::io::Stdout;
use sublime_fuzzy::best_match;
use unicode_width::UnicodeWidthChar;

use crate::domain::tmux_workspaces::aggregates::tmux::description::session::SessionDescription;
use crate::domain::tmux_workspaces::repositories::tmux::client_repository::{
    SwitchClientTarget, TmuxClientRepository,
};
use crate::utils::with_terminal;
use crate::{
    commands::command::RafaeltabCommand,
    domain::tmux_workspaces::repositories::tmux::{
        description_repository::SessionDescriptionRepository,
        session_repository::TmuxSessionRepository,
    },
};

#[derive(Default)]
pub struct TmuxSwitchCommand;

pub struct TmuxSwitchOptions<'a> {
    pub session_description_repository: &'a dyn SessionDescriptionRepository,
    pub session_repository: &'a dyn TmuxSessionRepository,
    pub client_repository: &'a dyn TmuxClientRepository,
}

impl RafaeltabCommand<TmuxSwitchOptions<'_>> for TmuxSwitchCommand {
    fn execute(
        &self,
        TmuxSwitchOptions {
            session_description_repository,
            session_repository,
            client_repository,
        }: TmuxSwitchOptions,
    ) {
        let descriptions = session_description_repository.get_session_descriptions();

        let res = fuzzy_pick(FuzzySearchArgs {
            items: &descriptions,
            search_text_fun: select_name,
        })
        .expect("Hey");

        if let Some(selected_session) = res {
            println!("You selected {}!", selected_session.name);
            let session = match &selected_session.session {
                Some(se) => se,
                None => &session_repository.new_session(selected_session),
            };

            client_repository.switch_client(None, SwitchClientTarget::Session(session));
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
                                // Delete one grapheme-aware-ish (width based) char
                                // Simple approach: remove last char
                                query.pop();
                                filtered = rebuild_filtered(&owned, &query);
                                selected_idx = 0;
                            }
                        }
                        KeyCode::Char(ch) => {
                            // Add printable characters
                            if !modifiers.contains(KeyModifiers::CONTROL)
                                && !modifiers.contains(KeyModifiers::ALT)
                            {
                                // Ignore wide control combinations
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

fn enter_tui(stdout: &mut Stdout) -> io::Result<()> {
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

fn leave_tui(stdout: &mut CrosstermBackend<Stdout>) -> io::Result<()> {
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
