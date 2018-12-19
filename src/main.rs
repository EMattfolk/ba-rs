extern crate i3ipc;
use i3ipc::I3Connection;
use i3ipc::I3EventListener;
use i3ipc::Subscription;

use std::time::SystemTime;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::time;

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

// Function to get a string representing the time
fn time () -> String
{
    let now = SystemTime::now();
    let duration = now.duration_since(SystemTime::UNIX_EPOCH).expect("Error");
    let secs = duration.as_secs() % (24 * 3600);
    let hour = (secs / 3600 + 1) % 24;
    let minute = (secs % 3600) / 60;

    time_string(hour) + &paint(":", BW_LIGHTERGREY, "F") + &time_string(minute)
}

// Function for getting the battery indcator
fn battery () -> String
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

// Function for getting the workspaces
fn workspaces () -> String
{
    let mut res = String::from("");
    let mut i3 = I3Connection::connect().unwrap();
    let spaces = i3.get_workspaces().unwrap().workspaces;
    for space in spaces {
        let mut space_string = String::from(" ") + &space.name + " ";
        if space.focused {
            space_string = paint(&space_string, BW_LIGHTGREY, "B");
        } 
        res += &space_string;
    }

    res
}

// Functions for getting wireless status
fn wireless () -> String
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

// Function for painting a string a certain color (not literally)
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

// Function for joining a vector of functions
fn join_fn (v: &Vec<impl Fn() -> String>, sep: &str) -> String
{
    let mut s = String::from("");
    if v.len() > 0 {
        s += &v[0]();
        for i in 1..v.len() {
            s += sep;
            s += &v[i]();
        }
    }

    s
}

// Function for printing the string to pipe to lemonbar
fn output_data (
    left:   &Vec<impl Fn() -> String>,
    center: &Vec<impl Fn() -> String>,
    right:  &Vec<impl Fn() -> String>
    )
{
    let begin     = "";
    let end       = " ";
    let separator = " ";

    let l = String::from("%{l}") + &join_fn(left, separator);
    let c = String::from("%{c}") + &join_fn(center, separator);
    let r = String::from("%{r}") + &join_fn(right, separator);

    println!("{}{}{}{}{}", begin, l, c, r, end);
}

fn main ()
{
    let l: Vec<fn() -> String> = vec![workspaces];
    let c: Vec<fn() -> String> = vec![time];
    let r: Vec<fn() -> String> = vec![wireless, battery];
    let l1 = l.clone();
    let c1 = c.clone();
    let r1 = r.clone();

    let mut listener = I3EventListener::connect().unwrap();
    listener.subscribe(&[Subscription::Workspace]).unwrap();

    thread::spawn(move || {

        let sleep_time = time::Duration::from_secs(2);

        loop {
            output_data(&l1, &c1, &r1);
            thread::sleep(sleep_time);
        }

    });

    for _event in listener.listen() {
        output_data(&l, &c, &r);
    }
}
