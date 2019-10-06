extern crate chrono;
use chrono::prelude::*;

extern crate i3ipc;
use i3ipc::reply::Node;
use i3ipc::reply::NodeType;
use i3ipc::I3Connection;
use i3ipc::I3EventListener;
use i3ipc::Subscription;

use std::env::args;
use std::fs::read_to_string;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

/*                                     */
/*             Created by              */
/*            Erik Mattfolk            */
/*               2018/12               */
/*                                     */
/*  A status bar data-generator-thing  */
/*         Designed to be light        */
/*                                     */
/*             -Features-              */
/*        No unecessary numbers        */
/*     Cool icons for applications     */
/*      Music player integration       */
/*         Minimalistic design         */
/*                                     */

/*               */
/* Configuration */
/*               */

// Update
const UPDATE_FREQ: u64 = 2;

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
const MU_PLAYERNAME: &str = "Spotify";
const MU_PLAYERICO: &str = "";
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

// Some icons for programs, in order of priority
const FIREFOX: &str = "";
const STEAM: &str = "";
const DISCORD: &str = "";
const CODE: &str = "";
const TERM: &str = "";
const UNDEF: &str = "";

// Window names (name, icon, starts_with)
//
// name: The name of the program (case sensitive)
// icon: The icon to indicate the program is running
// starts_with: Boolean representing if the name appears at the begining of the title
//     True  -> title starts with the name
//     False -> title ends with the name
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

/*         */
/* Structs */
/*         */

#[derive(Copy, Clone)]
enum ModuleData {
    Int(u64),
    TwoInt(u64, u64),
    Nil,
}

#[derive(Clone)]
struct Module {
    function: fn(&mut Module) -> String,
    data: ModuleData,
}

impl Module {
    fn create_string(&mut self) -> String {
        (self.function)(self)
    }
}

/*         */
/* Modules */
/*         */

// Module function to get a string representing the time
fn time(_data: &mut Module) -> String {
    let colon = paint(":", TI_COLON_COLOR, "F");
    let now = Local::now();

    format!("{:02}{}{:02}", now.hour(), colon, now.minute())
}

// Module function for getting the battery indcator
fn battery(_data: &mut Module) -> String {
    let capacity_path = String::from(BAT_PATH) + "capacity";
    let status_path = String::from(BAT_PATH) + "status";

    // Read capacity
    let capacity_string =
        read_to_string(capacity_path).expect("Unable to open battery capacity: Invalid path");

    let capacity: u32 = capacity_string.trim().parse().unwrap();

    // Read status
    let status = read_to_string(status_path).expect("Unable to open battery status: Invalid path");

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
fn workspaces(module: &mut Module) -> String {
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

            let stored_id = match module.data {
                ModuleData::Int(id) => id,
                _default => 0,
            };

            // Get node name
            let node_name = match node.name {
                Some(n) => {
                    if &n == MU_PLAYERNAME || stored_id == node.id as u64 {
                        module.data = ModuleData::Int(node.id as u64);
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
        module.data = ModuleData::Int(0);
    }

    space_strings.concat()
}

// Module function for getting network status
fn network(_data: &mut Module) -> String {
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
fn music(module: &mut Module) -> String {
    let mut i3 = I3Connection::connect().unwrap();
    let window_name: String;

    let stored_id = match module.data {
        ModuleData::Int(id) => id,
        _default => 0,
    };

    // Get window name and set data to the id of the window
    match get_node_from_name_or_id(i3.get_tree().unwrap(), MU_PLAYERNAME, stored_id) {
        Some(node) => {
            module.data = ModuleData::Int(node.id as u64);
            window_name = node.name.unwrap();
        }
        None => {
            module.data = ModuleData::Int(0);
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
fn cpu(module: &mut Module) -> String {
    // Read cpu values from /proc/stat
    let stats = read_to_string("/proc/stat")
        .expect("Failed to read procfile.")
        .split(" ")
        .skip(2)
        .take(7)
        .map(|x| x.parse().unwrap())
        .collect::<Vec<u64>>();

    let (last_idle, last_total) = match module.data {
        ModuleData::TwoInt(idle, total) => (idle, total),
        _default => (0, 0),
    };

    let (idle, total) = (stats[3], stats.iter().sum());

    module.data = ModuleData::TwoInt(idle, total);

    let idle_ratio = (idle - last_idle) as f64 / (total - last_total) as f64;

    (100 - (idle_ratio * 100.0).round() as u32).to_string()
}

/*                  */
/* Helper Functions */
/*                  */

// Search a tree for a node with a specified name or id
fn get_node_from_name_or_id(node: Node, name: &str, id: u64) -> Option<Node> {
    let node_name = match node.name.clone() {
        Some(n) => n,
        None => String::new(),
    };

    if node.id as u64 == id || node_name == name {
        return Some(node);
    }

    for n in node.nodes {
        let res = get_node_from_name_or_id(n, name, id);
        match res {
            Some(r) => return Some(r),
            None => {}
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

// Helper function for painting a string a certain color (not literally)
fn paint(string: &str, color: &str, layer: &str) -> String {
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

// Helper function for making buttons
fn buttonize(string: &str, command: &str) -> String {
    format!("%{{A:{1}:}}{0}%{{A}}", string, command)
}

// Helper function for joining a vector of modules
fn join_modules(modules: &mut Vec<Module>, sep: &str) -> String {
    modules
        .iter_mut()
        .map(|m| m.create_string())
        .collect::<Vec<String>>()
        .join(sep)
}

/*                   */
/* Program Functions */
/*                   */

// Program function for printing the string to pipe to lemonbar
fn output_data(modules: &mut (Vec<Module>, Vec<Module>, Vec<Module>)) {
    let begin = "";
    let end = " ";
    let separator = " ";

    let left = format!("%{{l}}{}", join_modules(&mut modules.0, separator));
    let center = format!("%{{c}}{}", join_modules(&mut modules.1, separator));
    let right = format!("%{{r}}{}", join_modules(&mut modules.2, separator));

    println!("{}{}{}{}{}", begin, left, center, right, end);
}

// Program function for sending lemonbar output as commands to i3
fn send_messages() {
    let mut connection = I3Connection::connect().expect("Failed to connect to i3");

    // Send messages to i3
    loop {
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read line");
        connection.run_command(&buffer).expect("I3 command failed");
    }
}

/*      */
/* Main */
/*      */

// The main function. This is where the magic happens
fn main() {
    // Get arguments
    let argv: Vec<String> = args().collect();

    if argv.contains(&String::from("--send")) {
        send_messages();
    }

    // Initialize modules
    let workspaces = Module {
        function: workspaces,
        data: ModuleData::Int(0),
    };
    let time = Module {
        function: time,
        data: ModuleData::Int(0),
    };
    let network = Module {
        function: network,
        data: ModuleData::Nil,
    };
    let battery = Module {
        function: battery,
        data: ModuleData::Nil,
    };
    let music = Module {
        function: music,
        data: ModuleData::Int(0),
    };
    let cpu = Module {
        function: cpu,
        data: ModuleData::TwoInt(0, 0),
    };

    // Arrange modules
    let left = vec![workspaces];
    let center = vec![time];
    let right = vec![cpu, music, network, battery];

    // Arcs used to share module data across threads
    let modules1 = Arc::new(Mutex::new((left, center, right)));
    let modules2 = modules1.clone();

    // Spawn a thread that updates bar every 2 seconds
    thread::spawn(move || {
        let sleep_time = time::Duration::from_secs(UPDATE_FREQ);

        loop {
            let mut modules = modules1.lock().unwrap();
            output_data(&mut modules);
            drop(modules);

            thread::sleep(sleep_time);
        }
    });

    // Set up i3 listener
    let mut listener = I3EventListener::connect().unwrap();
    listener.subscribe(&[Subscription::Workspace]).unwrap();

    for _event in listener.listen() {
        let mut modules = modules2.lock().unwrap();

        output_data(&mut modules);
    }
}
