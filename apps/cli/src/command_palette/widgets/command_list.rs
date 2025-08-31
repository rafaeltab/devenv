use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::layout::Spacing;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, BorderType, Borders, ListItem, ListState, Paragraph, WidgetRef,
};

use crate::command_palette::CommandPaletteCommand;

use super::my_list::{MyList, MyListItem};

pub struct CommandListWidget<'a> {
    pub commands: &'a Vec<Box<dyn CommandPaletteCommand>>,
    pub search_text: &'a str,
}

impl<'a> StatefulWidget for CommandListWidget<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State)
    where
        Self: Sized,
    {
        let block = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Reset))
            .title(" Commands ");

        let matcher = SkimMatcherV2::default();
        let mut scored_items: Vec<(&Box<dyn CommandPaletteCommand>, i64)> = self
            .commands
            .iter()
            .map(|x| {
                (
                    x,
                    matcher
                        .fuzzy_match(
                            &get_full_text_for_command(x.as_ref()).to_lowercase(),
                            &self.search_text.to_lowercase(),
                        )
                        .unwrap_or(0),
                )
            })
            .filter(|x| x.1 > 0 || self.search_text.trim().is_empty())
            .collect();

        scored_items.sort_by_key(|x| -x.1);

        let widgets: Vec<CommandListItemWidget> = scored_items
            .into_iter()
            .map(|x| CommandListItemWidget { command: x.0.as_ref() })
            .collect();
        let items: Vec<&dyn MyListItem> = widgets
            .iter()
            .map(|x| {
                let dynamic: &dyn MyListItem = x;
                dynamic
            })
            .collect();

        let list = MyList {
            block: Some(block),
            items,
        };

        list.render(area, buf);

        // let items: Vec<ListItem> = scored_items
        //     .iter()
        //     .map(|x| ListItem::from(CommandListItemWidget { command: x.0 }))
        //     .collect();
        //
        // let list = List::new(items).block(block);
        // StatefulWidget::render(list, area, buf, state)
    }
}

pub struct CommandListItemWidget<'a> {
    command: &'a dyn CommandPaletteCommand,
}

impl<'a> MyListItem for CommandListItemWidget<'a> {
    fn get_constraint(&self) -> Constraint {
        Constraint::Length(1)
    }
}

impl<'a> WidgetRef for CommandListItemWidget<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let title = self.command.get_title();
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(title.len().try_into().unwrap()),
                Constraint::Fill(1),
            ])
            .spacing(Spacing::Space(1))
            .split(area);
        Paragraph::new(self.command.get_icon()).render(layout[0], buf);
        Paragraph::new(self.command.get_title()).render(layout[1], buf);
        Paragraph::new(
            Line::from(self.command.get_description())
                .right_aligned()
                .fg(Color::DarkGray),
        )
        .render(layout[2], buf);
    }
}

impl<'a> From<CommandListItemWidget<'a>> for ListItem<'a> {
    fn from(value: CommandListItemWidget<'a>) -> Self {
        ListItem::new(vec![
            Line::from(vec![
                value.command.get_icon().into(),
                " ".into(),
                value.command.get_title().into(),
            ]),
            Line::from(value.command.get_description()).right_aligned(),
        ])
    }
}

fn get_full_text_for_command(command: &dyn CommandPaletteCommand) -> String {
    format!("{} {}", command.get_title(), command.get_description())
}
