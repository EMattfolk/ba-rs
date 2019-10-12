//! A small program used to create good-looking configurations
//! for lemonbar running on i3.

extern crate i3ipc;
use i3ipc::I3Connection;
use i3ipc::I3EventListener;
use i3ipc::Subscription;

use std::env::args;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use bardata::*;

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

const UPDATE_FREQ: u64 = 2;

/// Construct a string with the string representations
/// of all Modules in a Vector, separated by sep.
///
/// # Examples
///
/// ```
/// let out = join_modules(vec![time, cpu], " ");
///
/// println!("{}", out);
/// ```
pub fn join_modules(modules: &mut Vec<Module>, sep: &str) -> String {
    modules
        .iter_mut()
        .map(|m| m.create_string())
        .collect::<Vec<String>>()
        .join(sep)
}

/// Output a tuple of three Vectors of Modules to stout.
/// Modules are aligned in the following order:
///
/// left                center                right
///
/// Modules on each side are separated with a separator
pub fn output_data(modules: &mut (Vec<Module>, Vec<Module>, Vec<Module>)) {
    let begin = "";
    let end = " ";
    let separator = " ";

    let left = format!("%{{l}}{}", join_modules(&mut modules.0, separator));
    let center = format!("%{{c}}{}", join_modules(&mut modules.1, separator));
    let right = format!("%{{r}}{}", join_modules(&mut modules.2, separator));

    println!("{}{}{}{}{}", begin, left, center, right, end);
}

/// Send messages to i3.
/// Once called, the program will accept messages through stdin and
/// then send the messages directly to i3 until the program is closed.
pub fn send_messages() {
    let mut connection = I3Connection::connect().expect("Failed to connect to i3");

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
    let workspaces = Module::new(workspaces, ModuleData::Int(0));
    let time = Module::new(time, ModuleData::Int(0));
    let network = Module::new(network, ModuleData::Nil);
    let battery = Module::new(battery, ModuleData::Nil);
    let music = Module::new(music, ModuleData::Int(0));
    let cpu = Module::new(cpu, ModuleData::TwoInt(0, 0));

    // Arrange modules
    let left = vec![workspaces];
    let center = vec![time];
    let right = vec![music, cpu, network, battery];

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
