#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum Type {
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

impl From<usize> for Type {
    fn from(num: usize) -> Self {
        match num {
            0 => Type::RunCommand,
            1 => Type::Workspaces,
            2 => Type::Subscribe,
            3 => Type::Outputs,
            4 => Type::Tree,
            5 => Type::Marks,
            6 => Type::BarConfig,
            7 => Type::Version,
            8 => Type::BindingModes,
            9 => Type::Config,
            10 => Type::Tick,
            11 => Type::Sync,
            _ => panic!("Unknown message type found"),
        }
    }
}
