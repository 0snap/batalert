# Bat-Alert

An application to send [D-Bus](https://www.freedesktop.org/wiki/Software/dbus/) notifications when the (Linux) host's battery runs low.

## Prerequisites

- Install `libnotify`
- Install a Notification Server to render desktop notifications
  - Use [mako](https://github.com/emersion/mako) on Wayland/Sway setups

## Usage

You can run `batalert` with default setting to send the first notification when the battery falls below 15% and repeat the notification every 3%. `batalert` resets when you plug-in your charger.

### Customization

- Check the help text:
  ```
  batalert -h
  ```
- Show a notification when the battery falls below 20%, repeat notification every 4% on 16%, 12%, 8% ... :
  ```
  batalert --threshold 20 --notification-step 4
  ```
- Use a custom icon:
  ```
  batalert --icon /usr/share/icons/<your-icon>.png
  ```
- Monitor a certain battery, e.g., `BAT2`:
  ```
  batalert --uevent /sys/class/power_supply/BAT2/uevent
  ```




