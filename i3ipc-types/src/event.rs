//! For subscribing and receiving events, each struct matches a particular
//! `Subscribe` variant. For instance, subscribing with `Subscribe::Workspace`
//! will net `Event::Workspace` when workspace events are sent over the ipc.
use serde::{Deserialize, Serialize};

use crate::reply;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Subscribe {
    Workspace,
    Output,
    Mode,
    Window,
    #[serde(rename = "barconfig_update")]
    BarConfigUpdate,
    Binding,
    Shutdown,
    Tick,
}

impl From<u32> for Subscribe {
    fn from(num: u32) -> Self {
        match num {
            0 => Subscribe::Workspace,
            1 => Subscribe::Output,
            2 => Subscribe::Mode,
            3 => Subscribe::Window,
            4 => Subscribe::BarConfigUpdate,
            5 => Subscribe::Binding,
            6 => Subscribe::Shutdown,
            7 => Subscribe::Tick,
            _ => panic!("Unknown event found"),
        }
    }
}

impl From<Subscribe> for u32 {
    fn from(evt: Subscribe) -> Self {
        match evt {
            Subscribe::Workspace => 0,
            Subscribe::Output => 1,
            Subscribe::Mode => 2,
            Subscribe::Window => 3,
            Subscribe::BarConfigUpdate => 4,
            Subscribe::Binding => 5,
            Subscribe::Shutdown => 6,
            Subscribe::Tick => 7,
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Workspace(Box<WorkspaceData>),
    Output(OutputData),
    Mode(ModeData),
    Window(Box<WindowData>),
    BarConfig(BarConfigData),
    Binding(BindingData),
    Shutdown(ShutdownData),
    Tick(TickData),
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Copy, Clone)]
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

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone, Copy)]
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
    pub input_code: isize,
    pub symbol: Option<String>,
    pub input_type: BindType,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BindType {
    Keyboard,
    Mouse,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone)]
pub struct ShutdownData {
    pub change: ShutdownChange,
}

#[derive(Deserialize, Serialize, Eq, Hash, PartialEq, Debug, Clone, Copy)]
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
