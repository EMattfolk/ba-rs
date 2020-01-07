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

//! A small program used to create good-looking configurations
//! for lemonbar running on i3.

extern crate i3ipc;
use i3ipc::I3Connection;
use i3ipc::I3EventListener;
use i3ipc::Subscription;
use i3ipc::event::Event;

use std::env::args;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use bardata::*;

const UPDATE_FREQ: u64 = 2;

/// Send messages to i3.
/// Once called, the program will accept messages through stdin and
/// then send the messages directly to i3 until the program is closed.
/// Used to create clickable buttons.
///
/// # Examples
///
/// ```sh
/// bardata | lemonbar -p | bardata --send
/// ```
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

/// The main function.
/// It constructs the bar and then launches a thread that updates
/// the bar regularly. The program then waits for events from i3
/// which should update the bar.
fn main() {

    let argv: Vec<String> = args().collect();

    if argv.contains(&String::from("--send")) {
        send_messages();
    }

    // Initialize modules
    let workspaces = barfn!(workspaces);
    let time = barfn!(time);
    let network = barfn!(network);
    let battery = barfn!(battery);
    let music = barfn!(music);
    let cpu = barfn!(cpu);

    // Arrange modules
    let bar = Bar::new(
        vec![workspaces],
        vec![time],
        vec![music, cpu, network, battery]
    );

    // Prepare bar for multithreading
    let bar_loop = Arc::new(Mutex::new(bar));
    let bar_i3 = bar_loop.clone();

    // Spawn a thread that updates bar every 2 seconds
    thread::spawn(move || {
        let sleep_time = time::Duration::from_secs(UPDATE_FREQ);

        loop {
            bar_loop.lock().unwrap().output_data();
            thread::sleep(sleep_time);
        }
    });

    // Set up i3 listener
    let mut listener = I3EventListener::connect().unwrap();
    listener.subscribe(&[Subscription::Workspace, Subscription::Binding]).unwrap();

    for event in listener.listen() {
        match event.unwrap() {
            Event::WorkspaceEvent(_) => bar_i3.lock().unwrap().output_data(),
            Event::BindingEvent(e) => {
                let mut bar_i3 = bar_i3.lock().unwrap();
                if e.binding.command == "exec #show_bar_detail" {
                    if bar_i3.is_detailed() { continue; }
                    bar_i3.set_detailed(true);
                } else if e.binding.command == "exec #hide_bar_detail" {
                    if !bar_i3.is_detailed() { continue; }
                    bar_i3.set_detailed(false);
                }

                bar_i3.output_data()
            },
            _ => unreachable!()
        }
    }
}
