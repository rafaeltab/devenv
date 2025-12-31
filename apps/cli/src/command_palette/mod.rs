pub mod tui;
mod widgets;

pub struct CommandCtx {}

impl CommandCtx {
    fn prompt_select(&mut self, _prompt: SelectPrompt) -> String {
        todo!();
    }
    fn prompt_text<TValidate: Fn(String) -> Result<(), String>>(
        &mut self,
        _prompt: TextPrompt<TValidate>,
    ) -> String {
        todo!();
    }
}

struct TextPrompt<TValidate>
where
    TValidate: Fn(String) -> Result<(), String>,
{
    pub validate: TValidate,
    pub title: String,
}

struct SelectPrompt {
    pub title: String,
    pub options: Vec<SelectPromptOption>,
}

struct SelectPromptOption {
    pub value: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

pub trait CommandPaletteCommand {
    fn get_title(&self) -> &str;
    fn get_description(&self) -> &str;
    fn get_icon(&self) -> &str;
    fn execute(&mut self, ctx: &mut CommandCtx);
}

pub struct CommandBuilder {}

impl CommandBuilder {
    pub fn new() -> Self {
        CommandBuilder {}
    }
}

struct ExampleCommand {
    title: String,
    description: String,
}

impl CommandPaletteCommand for ExampleCommand {
    fn get_title(&self) -> &str {
        &self.title
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    fn execute(&mut self, _ctx: &mut CommandCtx) {
        todo!()
    }

    fn get_icon(&self) -> &str {
        "T"
    }
}
