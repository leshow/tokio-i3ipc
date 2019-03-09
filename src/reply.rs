use std::collections::HashMap;
// TODO manually impl Eq
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
struct Node {
    id: i32,
    name: Option<String>,
    #[serde(rename = "type")]
    node_type: NodeType,
    output: Option<String>,
    orientation: NodeOrientation,
    border: NodeBorder,
    percent: Option<f64>,
    rect: Rect,
    window_rect: Rect,
    deco_rect: Rect,
    geometry: Rect,
    window_properties: Option<HashMap<WindowProperty, Option<String>>>,
    urgent: bool,
    focused: bool,
    focus: Vec<i64>,
    sticky: bool,
    floating_nodes: Vec<Node>,
    nodes: Vec<Node>,
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
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Hash, Debug)]
#[serde(rename_all = "snake_case")]
enum NodeType {
    Root,
    Output,
    Con,
    FloatingCon,
    Workspace,
    Dockarea,
}
#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Debug, Clone)]
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
