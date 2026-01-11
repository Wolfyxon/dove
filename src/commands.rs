use crate::app::App;

pub const COMMAND_PREFIX: &str = "/";

pub struct CommandContext {
    pub args: Vec<String>
}

#[derive(Clone)]
pub struct ChatCommand {
    pub aliases: Vec<String>,
    pub description: String,
    handler: fn(&mut App, CommandContext) -> ()
}

impl ChatCommand {
    pub fn one_alias(alias: impl Into<String>) -> Self {
        Self {
            aliases: vec![alias.into()],
            description: "".to_string(),
            handler: |_, _| panic!("Command handler not implemented")
        }
    }

    pub fn with_handler(&mut self, handler: fn(&mut App, CommandContext) -> ()) -> Self {
        self.handler = handler;
        self.to_owned()
    }

    pub fn with_description(&mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self.to_owned()
    }

    pub fn execute(&self, app: &mut App, ctx: CommandContext) {
        (self.handler)(app, ctx);
    }
}
