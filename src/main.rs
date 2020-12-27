use clap::{App, Arg};
use crossbeam_channel::{select, tick};
use notify_rust::{Notification, Urgency};
use std::fs;
use std::time::Duration;

fn main() {
    // parse CLI options
    let matches = App::new("batalert")
        .version("0.2.0")
        .author("Felix Ortmann <flx.ortmann@gmail.com>")
        .about("Sends D-Bus notification when battery runs low.")
        .arg(
            Arg::with_name("uevent")
                .short("u")
                .long("uevent")
                .default_value("/sys/class/power_supply/BAT0/uevent")
                .help("Read the battery capacity from this uevent file."),
        )
        .arg(
            Arg::with_name("alert")
                .short("a")
                .long("alert")
                .default_value("15")
                .help("Send the first notification when battery falls below this threshold"),
        )
        .arg(
            Arg::with_name("notification-step")
                .short("n")
                .long("notification-step")
                .default_value("3")
                .help("Repeat notifications for every n percent the battery discharges"),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .default_value("15")
                .help("Hide the notification after t seconds"),
        )
        .arg(
            Arg::with_name("icon")
                .short("i")
                .long("icon")
                .default_value("/usr/share/icons/Adwaita/256x256/legacy/battery-caution.png")
                .help("Use this icon (PNG) when displaying notifications"),
        )
        .get_matches();
    let uevt_file = matches.value_of("uevent").unwrap();
    let icon = matches.value_of("icon").unwrap();
    let threshold = matches.value_of("alert").unwrap().parse::<i8>().unwrap();
    let step = matches
        .value_of("notification-step")
        .unwrap()
        .parse::<i8>()
        .unwrap();
    let timeout = matches.value_of("timeout").unwrap().parse::<i32>().unwrap();

    // periodic task to compare battery level with configuration
    let notify = |cap| send_notification(cap, icon, timeout * 1000);
    watch(&uevt_file, threshold, step, notify);
}

// Puts a notification to the D-Bus
fn send_notification(cap: i8, icon: &str, timeout: i32) {
    Notification::new()
        .summary(&format!("Battery {}%", cap))
        .body("Charge your battery soon to avoid shutdown")
        .icon(icon)
        .urgency(Urgency::Critical)
        .timeout(timeout)
        .show()
        .unwrap();
}

// Checks the current battery percentage. Calls the notification function
// periodically in the configured `step` interval if the battery discharges and
// is below the configured threshold
fn watch<F: Fn(i8)>(uevt_file: &str, threshold: i8, step: i8, notify: F) {
    let ticker = tick(Duration::from_millis(5000));
    let mut alert_threshold = threshold;
    loop {
        select! {
            recv(ticker) -> _ => {
                let (cap, status) = extract_status(&uevt_file);
                if status.to_lowercase() == "discharging" && cap <= alert_threshold {
                    notify(cap);
                    alert_threshold = cap - step;
                }
                else if status.to_lowercase() == "charging" {
                    alert_threshold = threshold;
                }
            }
        }
    }
}

// Extracts the battery capacity (percentage) and charging status
fn extract_status(uevt_file: &str) -> (i8, String) {
    let contents = fs::read_to_string(uevt_file).expect("Something went wrong reading the file");

    let cap = contents
        .lines()
        .find(|s| s.contains("POWER_SUPPLY_CAPACITY"))
        .unwrap()
        .split("=")
        .nth(1)
        .unwrap()
        .parse::<i8>()
        .unwrap();

    let status = contents
        .lines()
        .find(|s| s.contains("POWER_SUPPLY_STATUS"))
        .unwrap()
        .split("=")
        .nth(1)
        .unwrap()
        .to_string();
    return (cap, status);
}
