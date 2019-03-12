#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum Msg {
    RunCommand,
    Workspaces,
    Subscribe,
    Outputs,
    Tree,
    Marks,
    BarConfig,
    Version,
    BindingModes,
    Config,
    Tick,
    Sync,
}

impl From<usize> for Msg {
    fn from(num: usize) -> Self {
        match num {
            0 => Msg::RunCommand,
            1 => Msg::Workspaces,
            2 => Msg::Subscribe,
            3 => Msg::Outputs,
            4 => Msg::Tree,
            5 => Msg::Marks,
            6 => Msg::BarConfig,
            7 => Msg::Version,
            8 => Msg::BindingModes,
            9 => Msg::Config,
            10 => Msg::Tick,
            11 => Msg::Sync,
            _ => panic!("Unknown message type found"),
        }
    }
}
