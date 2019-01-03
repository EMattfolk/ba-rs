extern crate i3ipc;
use i3ipc::I3Connection;
use i3ipc::I3EventListener;
use i3ipc::Subscription;
use i3ipc::reply::Node;
use i3ipc::reply::NodeType;

use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::process::Command;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
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

// Some colors from badwolf
const BW_WHITE: &str       = "#f8f6f2";
const BW_DARK: &str        = "#121212";
const BW_GREY: &str        = "#35322d";
const BW_LIGHTGREY: &str   = "#45413b";
const BW_LIGHTERGREY: &str = "#857f78";
const BW_RED: &str         = "#ff2c4b";
const BW_GREEN: &str       = "#aeee00";
const BW_LIGHTBROWN: &str  = "#f4cf86";
const BW_ORANGE: &str      = "#ffa724";

// Default colors (these are also defined in the start script)
const BACKGROUND: &str     = BW_DARK;
const TEXT_COLOR: &str     = BW_WHITE;

// Network
const WL_PATH: &str        = "/sys/class/net/wlp3s0/";
const WL_IND: &str         = "";
const ETH_PATH: &str       = "/sys/class/net/enp2s0/";
const ETH_IND: &str        = "";
const NET_UP_COLOR: &str   = BW_GREEN;
const NET_DOWN_COLOR: &str = BW_RED;

// Battery
const BAT_PATH: &str       = "/sys/class/power_supply/BAT0/";
const BAT_IND: &str        = "";
const BAT_CHARGING: &str   = "";
const BAT_THRESHOLDS: [u32; 5] = [0, 20, 35, 50, 90];
const BAT_COLORS: [&str; 5]    = [BW_RED, BW_ORANGE, BW_LIGHTBROWN, TEXT_COLOR, BW_GREEN];

// Music
const MU_PLAYERNAME: &str  = "Spotify";
const MU_PLAYERICO: &str   = "";
const MU_IND: &str         = "";
const MU_IDLE_COLOR: &str  = BW_LIGHTGREY;
const MU_PLAY_COLOR: &str  = BW_ORANGE;

// Time
const TI_COLON_COLOR: &str = BW_LIGHTERGREY;

// Some icons for programs, in order of priority
const FIREFOX: &str        = "";
const STEAM: &str          = "";
const DISCORD: &str        = "";
const CODE: &str           = "";
const TERM: &str           = "";
const UNDEF: &str          = "";

// Window names (name, icon, starts_with)
//
// name: The name of the program (case sensitive)
// icon: The icon to indicate the program is running
// starts_with: Boolean representing if the name appears at the begining of the title
//     True  => title starts with the name
//     False => title ends with the name
const W_NAMES: [(&str, &str, bool); 4] = [
    ("nvim",    CODE,    true),
    ("Discord", DISCORD, false),
    ("Steam",   STEAM,   true),
    ("Firefox", FIREFOX, false),
];

// Change this to false if your workspace names are not numbered 1..10
const WS_NUMBERS: bool = true;


/*         */
/* Modules */
/*         */

// Module to get a string representing the time
fn time (data: &mut u64) -> String
{
    let now = SystemTime::now();
    let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let secs = (duration.as_secs() + *data) % (24 * 3600);
    let hour = (secs / 3600) % 24;
    let minute = (secs % 3600) / 60;

    time_string(hour) + &paint(":", TI_COLON_COLOR, "F") + &time_string(minute)
}

// Module for getting the battery indcator
fn battery (_data: &mut u64) -> String
{
    let capacity_path = String::from(BAT_PATH) + "capacity";
    let status_path   = String::from(BAT_PATH) + "status";

    // Read capacity
    let mut f = File::open(capacity_path).unwrap();
    let mut cap = String::new();
    f.read_to_string(&mut cap).unwrap();

    let cap: u32 = cap.trim().parse().unwrap();

    // Read status
    let mut f = File::open(status_path).unwrap();
    let mut stat = String::new();
    f.read_to_string(&mut stat).unwrap();

    let icon = if stat.trim() == "Discharging" { BAT_IND } else { BAT_CHARGING };

    // Assign color depending on capacity
    for t in (0..BAT_THRESHOLDS.len()).rev() {
        if cap >= BAT_THRESHOLDS[t] {
            return paint(icon, BAT_COLORS[t], "F");
        }
    }

    String::from("I you see this something went very wrong")
}

// Module for getting the workspaces
fn workspaces (data: &mut u64) -> String
{
    // Vector of strings to display
    let mut space_strings = Vec::with_capacity(10);
    // The connection to i3. Used to get data
    let mut i3 = I3Connection::connect().unwrap();
    // Vector of workspaces
    let mut spaces = Vec::with_capacity(11);
    // Find all workspaces
    get_workspaces_rec(&mut spaces, i3.get_tree().unwrap());

    // The current workspace
    let mut current_ws = 1;
    // Flag for if we find a music player window
    let mut music_found = false;
    // Create string from workspaces
    for space in spaces {

        // Dont include workspaces with zero width
        if space.rect.2 == 0 { continue; }

        let mut symbol_index: usize = 0;

        let space_name = space.name.clone().unwrap();
        let mut space_string = paint(&space_name, BW_LIGHTERGREY, "F");

        let mut focused = space.focused;

        let mut nodes = Vec::with_capacity(5);
        get_nodes_rec(&mut nodes, space);

        for node in nodes {

            focused |= node.focused;

            // Get node_name and give the music player highest priority
            let node_name = match node.name {
                Some(n) => 
                    if &n == MU_PLAYERNAME || *data == node.id as u64 {
                        space_string = String::from(MU_PLAYERICO);
                        *data = node.id as u64;
                        music_found = true;
                        symbol_index = W_NAMES.len();
                        continue;
                    }
                    else {
                        n
                    },
                None =>
                    String::from("")
            };

            // Match the window to the correct name in order of priority
            for i in (0..W_NAMES.len()).rev() {

                if i < symbol_index { break; }

                let (w_name, icon, starts_with) = W_NAMES[i];

                if (starts_with && node_name.starts_with(w_name)) ||
                    (!starts_with && node_name.ends_with(w_name))
                {
                    space_string = String::from(icon);
                    symbol_index = i + 1;
                }
            }

            // If we don't have an icon for the window
            if symbol_index == 0 && &space_string != TERM {
                space_string = String::from(UNDEF);
                let name_parts: Vec<&str> = node_name.split(" ").collect();
                if name_parts.len() > 1 && Path::new(name_parts[1]).exists() {
                    space_string = String::from(TERM);
                }
            }
        }

        space_string = String::from(" ") + &space_string + " ";

        if focused {
           space_string = paint(&space_string, BW_GREY, "B");
        } 

        // Show workspaces wedged between other workspaces
        if WS_NUMBERS {
            let n = space_name.parse().unwrap();
            if n > current_ws {
                for i in current_ws..n {
                    let name = &paint(&i.to_string(), BW_LIGHTGREY, "F");
                    space_strings.push(String::from(" ") + name + " ");
                }
                current_ws = n;
            }
            current_ws += 1;
        }

        space_strings.push(space_string);
    }

    // Reset data if there is no music player running
    if !music_found { *data = 0; }

    let mut res = String::with_capacity(200);

    for s in space_strings { res += &s; }

    res
}

// Module for getting network status
fn network (_data: &mut u64) -> String
{
    // Read the operstate file to see if the wireless is up
    let status_path = String::from(WL_PATH) + "operstate";
    let mut file = File::open(status_path).unwrap();
    let mut status = String::from("");

    file.read_to_string(&mut status).expect("Unable to read file");

    if status.trim() == "up" {
        return paint(WL_IND, NET_UP_COLOR, "F");
    }

    // Read the operstate file to see if the ethernet is up
    let status_path = String::from(ETH_PATH) + "operstate";
    let mut file = File::open(status_path).unwrap();
    let mut status = String::from("");

    file.read_to_string(&mut status).expect("Unable to read file");

    if status.trim() == "up" {
        paint(ETH_IND, NET_UP_COLOR, "F")
    }
    else {
        paint(WL_IND, NET_DOWN_COLOR, "F")
    }
}

// Module for getting music info
fn music (data: &mut u64) -> String
{
    // The connection to i3. Used to get data
    let mut i3 = I3Connection::connect().unwrap();
    // The window name
    let window_name: String;
    // Get music window if it exists
    match get_node_from_name_or_id(i3.get_tree().unwrap(), MU_PLAYERNAME, *data) {
        Some(idname) => { *data = idname.0; window_name = idname.1;},
        None => { *data = 0; return paint(MU_IND, BW_LIGHTGREY, "F") }
    }
    // Return if no music is playing
    if &window_name == MU_PLAYERNAME {
        return paint(MU_IND, MU_IDLE_COLOR, "F")
    }
    // Get name parts
    let name_parts: Vec<&str> = window_name.split(" - ").collect();

    paint(MU_IND, MU_PLAY_COLOR, "F") + " " + name_parts[0] + " - " + name_parts[1]
}


/*                  */
/* Helper Functions */
/*                  */

// Helper function for tree traversal to find a node with the given name
fn get_node_from_name_or_id (node: Node, name: &str, id: u64) -> Option<(u64, String)>
{
    let node_name = match node.name {
        Some(n) => n,
        None => String::from("")
    };

    if node.id as u64 == id || node_name == name {
        return Some((node.id as u64, node_name))
    }

    for n in node.nodes {
        let res = get_node_from_name_or_id(n, name, id);
        if res != None { return res; }
    }

    None
}

// Helper function for tree traversal to find workspaces
fn get_workspaces_rec (data: &mut Vec<Node>, node: Node)
{
    if node.nodetype == NodeType::Workspace {
        data.push(node);
    }
    else {
        for n in node.nodes {
            get_workspaces_rec(data, n);
        }
    }
}

// Helper function for tree traversal to find nodes
fn get_nodes_rec (data: &mut Vec<Node>, node: Node)
{
    if node.nodes.len() == 0 &&
       node.floating_nodes.len() == 0 &&
       node.nodetype != NodeType::Workspace
    {
        data.push(node);
    }
    else {
        for n in node.nodes {
            get_nodes_rec(data, n);
        }
        for n in node.floating_nodes {
            get_nodes_rec(data, n);
        }
    }
}

// Helper function for painting a string a certain color (not literally)
fn paint (string: &str, color: &str, to_paint: &str) -> String
{
    let mut painted = String::from(string);

    let default_color = 
        if to_paint == "F" { TEXT_COLOR }
        else {
            if to_paint == "U" {
                painted = String::from("%{+u}") + &painted + "&{-u}"; 
            }
            BACKGROUND
        };

    String::from("%{")+to_paint+color+"}"+&painted+"%{"+to_paint+default_color+"}"
}

// Helper fuction for getting properly formatted time
fn time_string (time: u64) -> String
{
    if time < 10 {
        String::from("0") + &time.to_string()
    }
    else {
        time.to_string()
    }

}

// Helper function for joining a vector of modules
fn join_module (m: &mut Vec<Module>, sep: &str) -> String
{
    if m.len() > 0 {
        let mut s = (m[0].function)(&mut m[0].data);
        for i in 1..m.len() {
            s += sep;
            s += &(m[i].function)(&mut m[0].data);
        } s
    }
    else {
        String::from("")
    }
}

// Helper function for getting time zone offset in seconds
fn get_tz_offset_in_seconds () -> u64
{
    // Get current time zone
    let tz = String::from_utf8(
        Command::new("date")
        .arg("+%z")
        .output()
        .expect("Error getting time zone")
        .stdout).unwrap();

    // Convert time zone to seconds
    let mut chars = tz.chars();
    let sign = chars.next().unwrap();
    let h1 = chars.next().unwrap().to_digit(10).unwrap();
    let h2 = chars.next().unwrap().to_digit(10).unwrap();
    let m1 = chars.next().unwrap().to_digit(10).unwrap();
    let m2 = chars.next().unwrap().to_digit(10).unwrap();

    let offset = 
        if sign == '-' {
            3600 * (24 - h1 * 10 - h2) - 60 * (m1 * 10 + m2)
        }
        else {
            3600 * (h1 * 10 + h2) + 60 * (m1 * 10 + m2)
        };

    offset as u64
}

/*                   */
/* Program Functions */
/*                   */

// Program function for printing the string to pipe to lemonbar
fn output_data (
    left:   &mut Vec<Module>,
    center: &mut Vec<Module>,
    right:  &mut Vec<Module>
    )
{
    let begin     = "";
    let end       = " ";
    let separator = " ";

    let l = String::from("%{l}") + &join_module(left, separator);
    let c = String::from("%{c}") + &join_module(center, separator);
    let r = String::from("%{r}") + &join_module(right, separator);

    println!("{}{}{}{}{}", begin, l, c, r, end);
}


/*         */
/* Structs */
/*         */

// Struct for handling modules
struct Module 
{
    function: fn(&mut u64) -> String,
    data: u64
}

impl Clone for Module
{
    fn clone(&self) -> Module
    {
        Module { function: self.function.clone(), data: self.data }
    }
}


/*      */
/* Main */
/*      */

// The main function. This is where the magic happens
fn main ()
{
    // Get timzone offset
    let o = get_tz_offset_in_seconds();

    // Initialize modules
    let workspaces = Module { function: workspaces, data: 0 };
    let time       = Module { function: time,       data: o };
    let network    = Module { function: network,    data: 0 };
    let battery    = Module { function: battery,    data: 0 };
    let music      = Module { function: music,      data: 0 };

    // Arrange modules
    let left   = vec![workspaces];
    let center = vec![time];
    let right  = vec![music, network, battery];

    // Arcs used to share module data across threads
    let l1 = Arc::new(Mutex::new(left));
    let c1 = Arc::new(Mutex::new(center));
    let r1 = Arc::new(Mutex::new(right));
    let l2 = l1.clone();
    let c2 = c1.clone();
    let r2 = r1.clone();

    // Spawn a thread that updates bar every 2 seconds
    thread::spawn(move || {

        let sleep_time = time::Duration::from_secs(2);

        loop {

            {
                let mut l = l1.lock().unwrap();
                let mut c = c1.lock().unwrap();
                let mut r = r1.lock().unwrap();

                output_data(&mut l, &mut c, &mut r);
            }

            thread::sleep(sleep_time);
        }

    });

    // Set up i3 listener
    let mut listener = I3EventListener::connect().unwrap();
    listener.subscribe(&[Subscription::Workspace]).unwrap();

    for _event in listener.listen() {

        let mut l = l2.lock().unwrap();
        let mut c = c2.lock().unwrap();
        let mut r = r2.lock().unwrap();

        output_data(&mut l, &mut c, &mut r);
    }
}
