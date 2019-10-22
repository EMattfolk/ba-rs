//! This library contains the main functionality for creating a bar.

extern crate chrono;
use chrono::prelude::*;

extern crate i3ipc;
use i3ipc::reply::Node;
use i3ipc::reply::NodeType;
use i3ipc::I3Connection;

use std::fs::read_to_string;

// Some icons for programs, in order of priority
const FIREFOX: &str = "";
const STEAM: &str = "";
const DISCORD: &str = "";
const CODE: &str = "";
const TERM: &str = "";
const UNDEF: &str = "";

const W_NAMES: [(&str, &str, bool); 7] = [
    ("", UNDEF, true),
    ("st", TERM, true),
    ("nvim", CODE, true),
    ("Discord", DISCORD, false),
    ("Steam", STEAM, true),
    ("Firefox", FIREFOX, false),
    (MU_PLAYERNAME, MU_PLAYERICO, true),
];

// Change this to false if your workspace names are not numbered 1..10
const WS_NUMBERS: bool = true;

// Some colors from badwolf
const BW_WHITE: &str = "#f8f6f2";
const BW_DARK: &str = "#121212";
const BW_GREY: &str = "#45413b";
const BW_LIGHTGREY: &str = "#857f78";
const BW_RED: &str = "#ff2c4b";
const BW_GREEN: &str = "#aeee00";
const BW_LIGHTBROWN: &str = "#f4cf86";
const BW_ORANGE: &str = "#ffa724";

// Default colors (these are also defined in the start script)
const BACKGROUND: &str = BW_DARK;
const TEXT_COLOR: &str = BW_WHITE;

// Network
const WL_PATH: &str = "/sys/class/net/wlp3s0/";
const WL_IND: &str = "";
const ETH_PATH: &str = "/sys/class/net/enp2s0/";
const ETH_IND: &str = "";
const NET_UP_COLOR: &str = BW_GREEN;
const NET_DOWN_COLOR: &str = BW_RED;

// Battery
const BAT_PATH: &str = "/sys/class/power_supply/BAT0/";
const BAT_IND: &str = "";
const BAT_CHARGING: &str = "";
const BAT_THRESHOLDS: [u32; 5] = [0, 20, 35, 50, 90];
const BAT_COLORS: [&str; 5] = [BW_RED, BW_ORANGE, BW_LIGHTBROWN, TEXT_COLOR, BW_GREEN];

// Music
pub const MU_PLAYERNAME: &str = "Spotify";
pub const MU_PLAYERICO: &str = "";
const MU_IND: &str = "";
const MU_IDLE_COLOR: &str = BW_GREY;
const MU_PLAY_COLOR: &str = BW_ORANGE;

// Time
const TI_COLON_COLOR: &str = BW_LIGHTGREY;

// Workspace
const WS_CURRENT: &str = BW_GREY;
const WS_NUM_COLOR: &str = BW_LIGHTGREY;

// Cpu
const CP_IND: &str = "";
const CP_THRESHOLDS: [u32; 5] = [0, 10, 20, 40, 80];
const CP_COLORS: [&str; 5] = [BW_GREEN, TEXT_COLOR, BW_LIGHTBROWN, BW_ORANGE, BW_RED];

#[macro_export]
macro_rules! barfn {
    ( $($f:expr)? ) => {
        $(Box::new(Module::new($f)))?
    };
}

/// A container which can print formated data for Lemonbar
///
/// # Examples
///
/// ```
/// use bardata::{Module, Bar, barfn};
///
/// fn updates(module: &mut Module<u64>) -> String {
///     module.data += 1;
///     (module.data - 1).to_string()
/// }
///
/// let mut bar = Bar {
///     left: vec![barfn!(updates)],
///     center: vec![],
///     right: vec![]
/// };
///
/// bar.output_data();
/// ```
pub struct Bar {
    pub left: Vec<Box<dyn BarStr>>,
    pub center: Vec<Box<dyn BarStr>>,
    pub right: Vec<Box<dyn BarStr>>,
}

impl Bar {
    /// Output a formatted string representing the content of the bar
    pub fn output_data(&mut self) {
        let begin = "";
        let end = " ";
        let separator = " ";

        let left = format!("%{{l}}{}", Bar::join_modules(&mut self.left, separator));
        let center = format!("%{{c}}{}", Bar::join_modules(&mut self.center, separator));
        let right = format!("%{{r}}{}", Bar::join_modules(&mut self.right, separator));

        println!("{}{}{}{}{}", begin, left, center, right, end);
    }

    /// Construct a string with the string representations
    /// of all Modules in a Vector, separated by sep.
    fn join_modules(modules: &mut Vec<Box<dyn BarStr>>, sep: &str) -> String {
        modules
            .iter_mut()
            .map(|m| m.create_string())
            .collect::<Vec<String>>()
            .join(sep)
    }
}

/// A container suited for making individual components on a `Bar`
///
/// It consists of two main components:
/// * A function returning the string to show on the bar
/// * A data field for storing data between updates
///
/// ```
/// use bardata::Module;
///
/// fn updates(module: &mut Module<u64>) -> String {
///     module.data += 1;
///     (module.data - 1).to_string()
/// }
///
/// let mut module = Module::new(updates);
///
/// assert_eq!(module.create_string(), "0");
/// assert_eq!(module.create_string(), "1");
/// assert_eq!(module.create_string(), "2");
/// ```
#[derive(Clone)]
pub struct Module<T> {
    function: fn(&mut Module<T>) -> String,
    pub data: T,
}

impl<T: Default> Module<T> {
    pub fn new(f: fn(&mut Module<T>) -> String) -> Module<T> {
        Module{function: f, data: Default::default()}
    }

    pub fn create_string(&mut self) -> String {
        (self.function)(self)
    }
}

pub trait BarStr: Send {
    fn create_string(&mut self) -> String;
}

impl<T: Send> BarStr for Module<T> {
    fn create_string(&mut self) -> String {
        (self.function)(self)
    }
}

/*         */
/* Modules */
/*         */

// Module function to get a string representing the time
pub fn time(_data: &mut Module<()>) -> String {
    let colon = paint(":", TI_COLON_COLOR, "F");
    let now = Local::now();

    format!("{:02}{}{:02}", now.hour(), colon, now.minute())
}

// Module function for getting the battery indcator
pub fn battery(_data: &mut Module<()>) -> String {
    let capacity_path = String::from(BAT_PATH) + "capacity";
    let status_path = String::from(BAT_PATH) + "status";

    // Read capacity
    let capacity_string =
        read_to_string(capacity_path)
        .expect("Unable to open battery capacity: Invalid path");

    let capacity: u32 = capacity_string.trim().parse().unwrap();

    // Read status
    let status = read_to_string(status_path)
        .expect("Unable to open battery status: Invalid path");

    let icon = if status.trim() == "Discharging" {
        BAT_IND
    } else {
        BAT_CHARGING
    };

    // Assign color depending on capacity
    for i in (0..BAT_THRESHOLDS.len()).rev() {
        if capacity >= BAT_THRESHOLDS[i] {
            return paint(icon, BAT_COLORS[i], "F");
        }
    }

    paint(icon, TEXT_COLOR, "F")
}

// Module function for getting the workspaces
pub fn workspaces(module: &mut Module<i64>) -> String {
    let mut i3 = I3Connection::connect().unwrap();
    let mut space_strings = Vec::with_capacity(10);
    let mut spaces = Vec::with_capacity(11);
    let mut current_ws = 1;
    let mut music_found = false;

    get_workspaces(&mut spaces, i3.get_tree().unwrap());

    for space in spaces {
        // Don't include workspaces with zero width
        if space.rect.2 == 0 {
            continue;
        }

        let space_name = space.name.clone().unwrap();
        let mut space_icon = space_name.clone();

        let mut focused = space.focused;

        let mut nodes = Vec::with_capacity(5);
        get_nodes(&mut nodes, space);

        let mut symbol_index: usize = 0;

        for node in nodes {
            focused |= node.focused;

            // Get node name
            let node_name = match node.name {
                Some(n) => {
                    if &n == MU_PLAYERNAME || module.data == node.id {
                        module.data = node.id;
                        music_found = true;
                        String::from(MU_PLAYERNAME)
                    } else {
                        n
                    }
                }
                None => String::new(),
            };

            // Match the window to the correct name in order of priority
            for i in (0..W_NAMES.len()).rev() {
                if i < symbol_index {
                    break;
                }

                let (w_name, icon, starts_with) = W_NAMES[i];

                if (starts_with && node_name.starts_with(w_name))
                    || (!starts_with && node_name.ends_with(w_name))
                {
                    space_icon = String::from(icon);
                    symbol_index = i + 1;
                }
            }
        }

        // Pad the icon with spaces
        space_icon = format!(" {} ", space_icon);

        // Create a button for easier navigation
        space_icon = buttonize(&space_icon, &format!("workspace {}", space_name));

        if focused {
            space_icon = paint(&space_icon, WS_CURRENT, "B");
        }

        // Show unused workspaces wedged between used workspaces
        if WS_NUMBERS {
            let n = space_name.parse().expect(
                "Workspace name is not a number. Please set WS_NUMBERS to\
                 false or change the name of your workspaces.",
            );
            if n > current_ws {
                for i in current_ws..n {
                    let mut name = format!(" {} ", i);
                    name = paint(&name, WS_NUM_COLOR, "F");
                    name = buttonize(&name, &format!("workspace {}", i));
                    space_strings.push(name);
                }
                current_ws = n;
            }
            current_ws += 1;
        }

        space_strings.push(space_icon);
    }

    if !music_found {
        module.data = 0;
    }

    space_strings.concat()
}

// Module function for getting network status
pub fn network(_data: &mut Module<()>) -> String {
    // Read the operstate file to see if the wireless is up
    let status_path = String::from(WL_PATH) + "operstate";
    let status = read_to_string(status_path).expect("Failed to read wireless status");

    if status.trim() == "up" {
        return paint(WL_IND, NET_UP_COLOR, "F");
    }

    // Read the operstate file to see if the ethernet is up
    let status_path = String::from(ETH_PATH) + "operstate";
    let status = read_to_string(status_path).expect("Failed to read wireless status");

    if status.trim() == "up" {
        paint(ETH_IND, NET_UP_COLOR, "F")
    } else {
        paint(WL_IND, NET_DOWN_COLOR, "F")
    }
}

// Module function for getting music info
pub fn music(module: &mut Module<i64>) -> String {
    let mut i3 = I3Connection::connect().unwrap();
    let window_name: String;

    // Get window name and set data to the id of the window
    match get_node_from_name_or_id(i3.get_tree().unwrap(), MU_PLAYERNAME, module.data) {
        Some(node) => {
            module.data = node.id;
            window_name = node.name.unwrap();
        }
        None => {
            module.data = 0;
            return paint(MU_IND, MU_IDLE_COLOR, "F");
        }
    }
    // Return if no music is playing
    if &window_name == MU_PLAYERNAME {
        return paint(MU_IND, MU_IDLE_COLOR, "F");
    }

    let name_parts: Vec<&str> = window_name.split(" - ").collect();

    assert!(name_parts.len() > 1, "Invalid song format");

    paint(MU_IND, MU_PLAY_COLOR, "F") + " " + name_parts[0] + " - " + name_parts[1]
}

// Module function for getting cpu info
pub fn cpu(module: &mut Module<(u64, u64)>) -> String {
    // Read cpu values from /proc/stat
    let stats = read_to_string("/proc/stat")
        .expect("Failed to read procfile.")
        .split(" ")
        .skip(2)
        .take(7)
        .map(|x| x.parse().unwrap())
        .collect::<Vec<u64>>();

    let (last_idle, last_total) = module.data;
    let (idle, total) = (stats[3], stats.iter().sum());

    module.data = (idle, total);

    let idle_ratio = (idle - last_idle) as f64 / (total - last_total) as f64;
    let load = 100 - (idle_ratio * 100.0).round() as u32;

    // Assign color depending on cpu load
    for i in (0..CP_THRESHOLDS.len()).rev() {
        if load >= CP_THRESHOLDS[i] {
            return paint(CP_IND, CP_COLORS[i], "F");
        }
    }

    paint(CP_IND, TEXT_COLOR, "F")
}

/*                  */
/* Helper Functions */
/*                  */

// Search a tree for a node with a specified name or id
fn get_node_from_name_or_id(node: Node, name: &str, id: i64) -> Option<Node> {
    let t = String::new();
    let node_name = node.name.as_ref().unwrap_or(&t);

    if node.id == id || node_name == name {
        return Some(node);
    }

    for n in node.nodes {
        if let Some(res) = get_node_from_name_or_id(n, name, id) {
            return Some(res);
        }
    }

    None
}

// Helper function for tree traversal to find workspaces
fn get_workspaces(data: &mut Vec<Node>, node: Node) {
    if node.nodetype == NodeType::Workspace {
        data.push(node);
    } else {
        for n in node.nodes {
            get_workspaces(data, n);
        }
    }
}

// Helper function for tree traversal to find nodes
fn get_nodes(data: &mut Vec<Node>, node: Node) {
    if node.nodes.len() == 0
        && node.floating_nodes.len() == 0
        && node.nodetype != NodeType::Workspace
    {
        data.push(node);
    } else {
        for n in node.nodes {
            get_nodes(data, n);
        }
        for n in node.floating_nodes {
            get_nodes(data, n);
        }
    }
}

/// Helper function for changing colors on the bar
pub fn paint(string: &str, color: &str, layer: &str) -> String {
    let mut to_paint = String::from(string);

    let default_color = if layer == "F" {
        TEXT_COLOR
    } else {
        if layer == "U" {
            to_paint = format!("%{{+u}}{}%{{-u}}", to_paint);
        }
        BACKGROUND
    };

    format!(
        "%{{{2}{1}}}{0}%{{{2}{3}}}",
        to_paint, color, layer, default_color
    )
}

/// Helper function for making lemonbar buttons
pub fn buttonize(string: &str, command: &str) -> String {
    format!("%{{A:{1}:}}{0}%{{A}}", string, command)
}