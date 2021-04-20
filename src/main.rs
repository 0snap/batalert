use clap::{App, Arg};
use crossbeam_channel::{select, tick};
use notify_rust::{Notification, Urgency};
use std::time::Duration;
use std::{fs, path, process};

#[tokio::main]
async fn main() {
    // parse CLI options
    let matches = App::new("batalert")
        .version("0.3.0")
        .author("Felix Ortmann <flx.ortmann@gmail.com>")
        .about("Sends D-Bus notification when battery runs low.")
        .arg(
            Arg::with_name("uevent")
                .short("u")
                .long("uevent")
                .takes_value(true)
                .multiple(true)
                .default_value("/sys/class/power_supply/BAT0/uevent")
                .help("Read the battery capacity from this uevent file."),
        )
        .arg(
            Arg::with_name("alert")
                .short("a")
                .long("alert")
                .takes_value(true)
                .default_value("15")
                .help("Send the first notification when battery falls below this threshold"),
        )
        .arg(
            Arg::with_name("notification-step")
                .short("n")
                .long("notification-step")
                .takes_value(true)
                .default_value("3")
                .help("Repeat notifications for every n percent the battery discharges"),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .takes_value(true)
                .default_value("15")
                .help("Hide the notification after t seconds"),
        )
        .arg(
            Arg::with_name("icon")
                .short("i")
                .long("icon")
                .takes_value(true)
                .default_value("/usr/share/icons/Adwaita/256x256/legacy/battery-caution.png")
                .help("Use this icon (PNG) when displaying notifications"),
        )
        .get_matches();
    let uevt_files = matches.values_of("uevent").unwrap();
    let icon = matches.value_of("icon").unwrap().to_owned();
    let threshold = matches.value_of("alert").unwrap().parse::<i8>().unwrap();
    let step = matches
        .value_of("notification-step")
        .unwrap()
        .parse::<i8>()
        .unwrap();
    let timeout = matches.value_of("timeout").unwrap().parse::<i32>().unwrap();

    let mut watchers = vec![];
    for uevt_file in uevt_files {
        if !path::Path::new(&uevt_file).exists() {
            println!("File {} does not exist", &uevt_file);
            process::exit(1);
        }
        // periodic task to compare battery level with configuration
        let ic = icon.clone();
        let file = uevt_file.to_owned().clone();
        watchers.push(tokio::spawn(async move {
            watch(&file, threshold, step, ic, timeout).await;
        }));
    }

    futures::future::join_all(watchers).await;
}

// Puts a notification to the D-Bus
fn send_notification(bat_name: String, cap: i8, icon: &str, timeout: i32) {
    match Notification::new()
        .summary(&format!("Battery {} - {}%", bat_name, cap))
        .body("Charge your battery soon to avoid shutdown")
        .icon(icon)
        .urgency(Urgency::Critical)
        .timeout(timeout)
        .show()
    {
        Ok(_) => (),
        Err(err) => println!("Error sending notification: {}", err),
    };
}

// Periodically checks the battery status and sends notifications to the user if
// the battery discharges and is below the configured threshold.
async fn watch(uevt_file: &str, threshold: i8, step: i8, icon: String, timeout: i32) {
    let ticker = tick(Duration::from_millis(5000));
    let mut alert_threshold = threshold;
    loop {
        select! {
            recv(ticker) -> _ => {
                let (name, cap, status) = extract_info(&uevt_file);
                if status.to_lowercase() == "discharging" && cap <= alert_threshold {
                    send_notification(name, cap, &icon, timeout * 1000);
                    alert_threshold = cap - step;
                }
                else if status.to_lowercase() == "charging" {
                    alert_threshold = threshold;
                }
            }
        }
    }
}

// Extracts the battery name, capacity (percentage), and charging status
fn extract_info(uevt_file: &str) -> (String, i8, String) {
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

    let bat_name = contents
        .lines()
        .find(|s| s.contains("POWER_SUPPLY_NAME"))
        .unwrap()
        .split("=")
        .nth(1)
        .unwrap()
        .to_string();
    return (bat_name, cap, status);
}
