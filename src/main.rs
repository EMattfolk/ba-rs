extern crate i3ipc;
use i3ipc::I3Connection;
use i3ipc::I3EventListener;
use i3ipc::Subscription;
use i3ipc::reply::Node;
use i3ipc::reply::NodeType;

use std::time::SystemTime;
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
/*       Designed to be light          */
/*                                     */
/*              Features               */
/*        No unecessary numbers        */
/*     Cool icons for applications     */
/*         Minimalistic design         */
/*                                     */

// Some colors from badwolf
const BW_LIGHTGREY: &str   = "#45413b";
const BW_LIGHTERGREY: &str = "#857f78";
const BW_RED: &str         = "#ff2c4b";
const BW_GREEN: &str       = "#aeee00";
const BW_LIGHTBROWN: &str  = "#f4cf86";
const BW_ORANGE: &str      = "#ffa724";

// Default colors
const BW_BACKGROUND: &str  = "#121212";
const BW_WHITE: &str       = "#f8f6f2";

// Wireless
const WL_PATH: &str        = "/sys/class/net/wlp3s0/";
const WL_IND: &str         = "";

// Battery
const BAT_PATH: &str       = "/sys/class/power_supply/BAT0/";
const BAT_IND: &str        = "";

// Battery colors
const BAT_THRESHOLDS: [u32; 5] = [0, 20, 35, 50, 90];
const BAT_COLORS: [&str; 5]    = [BW_RED, BW_ORANGE, BW_LIGHTBROWN, BW_WHITE, BW_GREEN];

// Some icons for programs, in order of priority
const SPOTIFY: &str = "";
const FIREFOX: &str = "";
const STEAM: &str   = "";
const DISCORD: &str = "";
const CODE: &str    = "";
const TERM: &str    = "";
const UNDEF: &str   = "";
const EMPTY: &str   = " ";

// Window names (name, icon, starts_with)
//
// name: The name of the program (case sensitive)
// icon: The icon to indicate the program is running
// starts_with: Boolean representing if the name appears at the begining of the title
//     True  => title starts with the name
//     False => title ends with the name
const W_NAMES: [(&str, &str, bool); 5] = [
    ("nvim",    CODE,    true),
    ("Discord", DISCORD, false),
    ("Steam",   STEAM,   true),
    ("Firefox", FIREFOX, false),
    ("Spotify", SPOTIFY, true)
];

//
// Modules
//

// TODO: UTF time support
// Module to get a string representing the time
fn time (data: &mut u64) -> String
{
    // TODO: A working time system
    let now = SystemTime::now();
    let duration = now.duration_since(SystemTime::UNIX_EPOCH).expect("Error");
    let secs = duration.as_secs() % (24 * 3600);
    let hour = (secs / 3600 + *data) % 24;
    let minute = (secs % 3600) / 60;

    time_string(hour) + &paint(":", BW_LIGHTERGREY, "F") + &time_string(minute)
}

// Module for getting the battery indcator
fn battery (_data: &mut u64) -> String
{
    let capacity_path = String::from(BAT_PATH) + "capacity";

    let mut f = File::open(capacity_path).expect("Battery not found");
    let mut cap = String::new();
    f.read_to_string(&mut cap).expect("Error reading file");

    let cap: u32 = cap.trim().parse().unwrap();

    for t in (0..BAT_THRESHOLDS.len()).rev() {
        if cap >= BAT_THRESHOLDS[t] {
            return paint(BAT_IND, BAT_COLORS[t], "F");
        }
    }

    String::from("I you see this something went very wrong")
}

// Module for getting the workspaces
fn workspaces (data: &mut u64) -> String
{
    // The string to return
    let mut res = String::from("");
    // The connection to i3. Used to get data
    let mut i3 = I3Connection::connect().unwrap();
    // Vector of workspaces
    let mut spaces = Vec::new();
    // Find all workspaces
    get_workspaces_rec(&mut spaces, i3.get_tree().unwrap());
    // Create string from workspaces
    for space in spaces {

        // Dont include workspaces with zero width
        if space.rect.2 == 0 { continue; }

        let mut symbol_index: usize = 0;
        let mut space_string = String::from(EMPTY);

        let mut focused = space.focused;
        for node in space.nodes {

            focused |= node.focused;

            // TODO: Add floating nodes
            let node_name = match node.name {
                Some(n) => n,
                None => String::from("")
            };

            // Spotify integration
            let name_parts: Vec<&str> = node_name.split(" ").collect();
            let id = node.id as u64;

            if node_name == "Spotify" {
                *data = id;
            }

            if name_parts.len() > 1 {
                if Path::new(name_parts[1]).exists() {
                    if symbol_index == 0 {
                        space_string = String::from(TERM);
                    }
                }
                else if *data == id {
                    // TODO: Shorten song names, split at " - "
                    symbol_index = W_NAMES.len();
                    space_string = String::from(SPOTIFY) + " " + &node_name;
                }
            }
            else if symbol_index == 0 {
                space_string = String::from(UNDEF);
            }

            // Match the window to the correct name
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
        }

        space_string = String::from(" ") + &space_string + " ";

        if focused {
           space_string = paint(&space_string, BW_LIGHTGREY, "B");
        } 

        res += &space_string;
    }

    res
}

// Module for getting wireless status
fn wireless (_data: &mut u64) -> String
{
    // Read the operstate file to see if the network is up
    let status_path = String::from(WL_PATH) + "operstate";
    let mut file = File::open(status_path).unwrap();
    let mut status = String::from("");

    file.read_to_string(&mut status).expect("Unable to read file");

    if status.trim() == "up" {
        paint(WL_IND, BW_GREEN, "F")
    }
    else {
        paint(WL_IND, BW_RED, "F")
    }
}

//
// Helper functions
//

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

// Helper function for painting a string a certain color (not literally)
fn paint (string: &str, color: &str, to_paint: &str) -> String
{
    let mut painted = String::from(string);

    let default_color = 
        if to_paint == "F" { BW_WHITE }
        else {
            if to_paint == "U" {
                painted = String::from("%{+u}") + &painted + "&{-u}"; 
            }
            BW_BACKGROUND
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

//
// Program functions
//

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

// The main function. This is where the magic happens
fn main ()
{
    let workspaces = Module { function: workspaces, data: 0 };
    let time       = Module { function: time,       data: 19 };
    let wireless   = Module { function: wireless,   data: 0 };
    let battery    = Module { function: battery,    data: 0 };

    // TODO: Add Arc for cross thread stuff
    let mut l: Vec<Module> = vec![workspaces];
    let mut c: Vec<Module> = vec![time];
    let mut r: Vec<Module> = vec![wireless, battery];
    let mut l1 = l.clone();
    let mut c1 = c.clone();
    let mut r1 = r.clone();

    let mut listener = I3EventListener::connect().unwrap();
    listener.subscribe(&[Subscription::Workspace]).unwrap();

    thread::spawn(move || {

        let sleep_time = time::Duration::from_secs(2);

        loop {
            output_data(&mut l1, &mut c1, &mut r1);
            thread::sleep(sleep_time);
        }

    });

    for _event in listener.listen() {
        output_data(&mut l, &mut c, &mut r);
    }
}
