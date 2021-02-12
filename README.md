<h1 align="center">
  batalert
</h1>
<h4 align="center">

Send [D-Bus](https://www.freedesktop.org/wiki/Software/dbus/) notifications when the (Linux) host's battery runs low.

[![License][license-badge]][license-url]

</h4>


## Prerequisites

- Install `libnotify`
- Install a Notification Server to render desktop notifications
  - Use [mako](https://github.com/emersion/mako) on Wayland/Sway setups

## Usage

You can run `batalert` with default setting to send the first notification when the battery falls below 15% and repeat the notification every 3%. Notifications timeout after 15 seconds. `batalert` resets when you plug-in your charger.

### Customization

- Check the help text:
  ```
  batalert -h
  ```
- Show a notification when the battery falls below 20%, repeat notification every 4% on 16%, 12%, 8% ... :
  ```
  batalert --alert 20 --notification-step 4
  ```
- Timeout notifications after a minute:
  ```
  batalert --timeout 60
  ```
- Use a custom icon:
  ```
  batalert --icon /usr/share/icons/<your-icon>.png
  ```
- Monitor a particular battery, e.g., `BAT2`:
  ```
  batalert --uevent /sys/class/power_supply/BAT2/uevent
  ```
- Monitor multiple batteries (repeat the `--uevent` option):
  ```
  batalert --uevent /sys/class/power_supply/BAT0/uevent --uevent /path/to/bat1 -u /path/to/bat2
  ```

## Building

Build the app via `Cargo`:
```
cargo build --release
```


[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-url]: https://github.com/0ortmann/batalert/blob/master/LICENSE