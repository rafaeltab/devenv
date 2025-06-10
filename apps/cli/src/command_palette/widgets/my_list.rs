use ratatui::{layout::{Constraint, Direction, Layout}, prelude::{BlockExt, Buffer, Rect}, widgets::{Block, Widget, WidgetRef}};

pub struct MyList<'a> {
    pub block: Option<Block<'a>>,
    pub items: Vec<&'a dyn MyListItem>
}

pub trait MyListItem: WidgetRef {
    fn get_constraint(&self) -> Constraint {
        Constraint::Fill(1)
    }
}

impl<'a> Widget for MyList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized {
        let mut constraints = vec![];
        for item in &self.items {
            constraints.push(item.get_constraint())
        }
        constraints.push(Constraint::Fill(100));

        self.block.render(area, buf);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(self.block.inner_if_some(area));

        for i in 0..self.items.len() {
            let item = &self.items[i];
            item.render_ref(layout[i], buf);
        }
    }
}
