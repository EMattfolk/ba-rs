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

// Battery path
const BAT_PATH: &str       = "/sys/class/power_supply/BAT0/";
const BAT_IND: &str        = "";

// Battery colors
const BAT_THRESHOLDS: [u32; 5] = [0, 20, 35, 50, 90];
const BAT_COLORS: [&str; 5]    = [BW_RED, BW_ORANGE, BW_LIGHTBROWN, BW_WHITE, BW_GREEN];

fn time () -> String
{
    let now = SystemTime::now();
    let duration = now.duration_since(SystemTime::UNIX_EPOCH).expect("Error");
    let secs = duration.as_secs() % (24 * 3600);
    let hour = (secs / 3600 + 1) % 24;
    let minute = (secs % 3600) / 60;

    time_string(hour) + &paint(":", BW_LIGHTERGREY, "F") + &time_string(minute)
}

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

fn time_string (time: u64) -> String
{
    if time < 10 {
        String::from("0") + &time.to_string()
    }
    else {
        time.to_string()
    }

}

fn get_formatted_string (
    left:   &Vec<impl Fn() -> String>,
    center: &Vec<impl Fn() -> String>,
    right:  &Vec<impl Fn() -> String>
    ) -> String
{
    let begin     = String::from("");
    let end       = " ";
    let separator = " ";

    // TODO: create a function to do this
    let mut l = String::from("%{l}");
    if left.len() > 0 {
        l += &left[0]();
        for i in 1..left.len() {
            l += separator;
            l += &left[i]();
        }
    }

    let mut c = String::from("%{c}");
    if center.len() > 0 {
        c += &center[0]();
        for i in 1..center.len() {
            c += separator;
            c += &center[i]();
        }
    }

    let mut r = String::from("%{r}");
    if right.len() > 0 {
        r += &right[0]();
        for i in 1..right.len() {
            r += separator;
            r += &right[i]();
        }
    }

    begin + &l + &c + &r + end
}

fn main ()
{
    let l: Vec<fn() -> String> = Vec::new();
    let c: Vec<fn() -> String> = vec![time];
    let r: Vec<fn() -> String> = vec![battery];

    let sleep_time = time::Duration::from_secs(2);

    loop {
        println!("{}", get_formatted_string(&l, &c, &r));
        thread::sleep(sleep_time);
    }
}
