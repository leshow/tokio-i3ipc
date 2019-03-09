use crate::reply;

// #[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
// pub enum Event {
//     Workspace(WorkspaceData), // consider boxing
//     // one interesting thing about the Event from i3ipc, because enums take the size
//     // of their largest variant, even small events take up a lot of space
//     Output(OutputData),
//     Mode(ModeData),
//     Window(WindowData),
//     BarConfigUpdate(BarConfigData),
//     Shutdown(ShutdownData),
//     Tick(TickData),
// }

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub enum Event {
    Workspace,
    Output,
    Mode,
    Window,
    BarConfigUpdate,
    Shutdown,
    Tick,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceChange {
    Focus,
    Init,
    Empty,
    Urgent,
    Rename,
    Reload,
    Restored,
    Move,
}

// TODO manually impl Eq
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct WorkspaceData {
    pub change: WorkspaceChange,
    pub current: Option<reply::Node>,
    pub old: Option<reply::Node>,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone)]
pub struct OutputData {
    pub change: String,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone)]
pub struct ModeData {
    pub change: String,
    pub pango_markup: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct WindowData {
    pub change: WindowChange,
    pub container: reply::Node,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum WindowChange {
    ///  the window has become managed by i3  
    New,
    /// the window has closed
    Close,
    /// the window has received input focus
    Focus,
    /// the window’s title has changed
    Title,
    /// the window has entered or exited fullscreen mode
    FullscreenMode,
    /// the window has changed its position in the tree
    Move,
    /// the window has transitioned to or from floating
    Floating,
    /// the window has become urgent or lost its urgent status
    Urgent,
    /// a mark has been added to or removed from the window
    Mark,
}

pub type BarConfigData = reply::BarConfig;

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone)]
pub struct BindingData {
    pub change: String,
    pub binding: BindingObject,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone)]
pub struct BindingObject {
    pub command: String,
    pub event_state_mask: Vec<String>,
    pub input_code: i32,
    pub symbol: Option<String>,
    pub input_type: BindType,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BindType {
    Keyboard,
    Mouse,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone)]
pub struct ShutdownData {
    pub change: ShutdownChange,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ShutdownChange {
    Restart,
    Exit,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone)]
pub struct TickData {
    pub first: bool,
    pub payload: String,
}
