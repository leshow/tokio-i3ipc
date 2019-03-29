use serde_derive::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use std::collections::HashMap;

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Hash, Debug)]
pub struct Success {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Hash, Debug)]
pub struct Workspace {
    pub num: usize,
    pub name: String,
    pub visible: bool,
    pub focused: bool,
    pub urgent: bool,
    pub rect: Rect,
    pub output: String,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Hash, Debug)]
pub struct Outputs {
    pub name: String,
    pub active: bool,
    pub primary: bool,
    pub current_workspace: Option<String>,
    pub rect: Rect,
}

/// Node reply
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
// TODO manually impl Eq
pub struct Node {
    pub id: usize,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub output: Option<String>,
    pub orientation: NodeOrientation,
    pub border: NodeBorder,
    pub percent: Option<f64>,
    pub rect: Rect,
    pub window_rect: Rect,
    pub deco_rect: Rect,
    pub geometry: Rect,
    pub window_properties: Option<HashMap<WindowProperty, Option<String>>>,
    pub urgent: bool,
    pub focused: bool,
    pub focus: Vec<i64>,
    pub sticky: bool,
    pub floating: Floating,
    pub floating_nodes: Vec<Node>,
    pub fullscreen_mode: FullscreenMode,
    pub nodes: Vec<Node>,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Floating {
    AutoOff,
    AutoOn,
    UserOff,
    UserOn,
}

#[derive(Deserialize_repr, Serialize_repr, Eq, PartialEq, Clone, Hash, Debug)]
#[repr(u8)]
pub enum FullscreenMode {
    None = 0,
    Output = 1,
    Global = 2,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WindowProperty {
    Title,
    Instance,
    Class,
    WindowRole,
    TransientFor,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Hash, Debug)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Hash, Debug)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Root,
    Output,
    Con,
    FloatingCon,
    Workspace,
    Dockarea,
}
#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum NodeBorder {
    Normal,
    None,
    Pixel,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum NodeLayout {
    SplitH,
    SplitV,
    Stacked,
    Tabbed,
    Dockarea,
    Output,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum NodeOrientation {
    Horizontal,
    Vertical,
    None,
}

/// Marks Reply
#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Clone)]
pub struct Marks(Vec<String>);

/// BarIds
#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Clone)]
pub struct BarIds(Vec<String>);

/// BarConfig Reply
#[derive(Deserialize, Serialize, Eq, PartialEq, Debug, Clone)]
pub struct BarConfig {
    pub id: String,
    pub mode: String,
    pub position: String,
    pub status_command: String,
    pub font: String,
    pub workspace_buttons: bool,
    pub binding_mode_indicator: bool,
    pub verbose: bool,
    pub colors: HashMap<BarPart, String>,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum BarPart {
    Background,
    Statusline,
    Separator,
    FocusedBackground,
    FocusedStatusline,
    FocusedSeparator,
    FocusedWorkspaceText,
    FocusedWorkspaceBg,
    FocusedWorkspaceBorder,
    ActiveWorkspaceText,
    ActiveWorkspaceBg,
    ActiveWorkspaceBorder,
    InactiveWorkspaceText,
    InactiveWorkspaceBg,
    InactiveWorkspaceBorder,
    UrgentWorkspaceText,
    UrgentWorkspaceBg,
    UrgentWorkspaceBorder,
    BindingModeText,
    BindingModeBg,
    BindingModeBorder,
}

/// Version reply
#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Clone)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub human_readable: String,
    pub loaded_config_file_name: String,
}

/// Binding Modes Reply
#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Clone)]
pub struct BindingModes(Vec<String>);

/// Config Reply
#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Clone)]
pub struct Config {
    pub config: String,
}
