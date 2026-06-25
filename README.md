# ajazz

A small Rust tool that syncs your Mac's current local time to an **Ajazz AK650** keyboard's on-board clock display over USB HID.
The motivation is that the official driver is Windows only.

It's a one-shot CLI program that looks for the device, pushes the current date/time, and exits. If the keyboard isn't connected it prints a message and exits cleanly.

## Disclaimer

The super basic reverse engineering was done by packet sniffing the official driver with Wireshark. I observed two "initialization" packets and one "cleanup" packet that encapsulated the time sync packet. I have no idea what those packets do but it seems in the official driver they're always sent exactly the same way so I replicated that behaviour here.

This may not work with other models. I will look into reverse engineering other aspects of the software (e.g. GIF upload).


## Run as a daemon (macOS)

To keep the clock in sync automatically, install `ajazz` as a `launchd`
**LaunchAgent**. It will run the moment the keyboard is attached and again on a
fixed interval while it stays connected. When the keyboard is unplugged, the
periodic runs simply find no device and exit.

```bash
./dist/install.sh
```

This will:

- Build the release binary (if needed) and copy it to `~/.local/bin/ajazz`.
- Render `dist/com.jasonyuan.ajazz.plist` with absolute paths into
  `~/Library/LaunchAgents/`.
- Load the agent into your GUI session.

Trigger a sync immediately:

```bash
launchctl kickstart -k gui/$(id -u)/com.jasonyuan.ajazz
```

Logs are written to `~/Library/Logs/ajazz/`.

To remove it:

```bash
./dist/uninstall.sh
```

### Tuning

- **Sync frequency:** edit `StartInterval` (seconds) in
  `dist/com.jasonyuan.ajazz.plist`, then re-run `./dist/install.sh`.
- **Input Monitoring permission:** macOS may require granting the binary access
  under System Settings → Privacy & Security → Input Monitoring before it can
  talk to the HID device. If syncs fail silently, check `ajazz.err.log` first.
- **USB-attach trigger:** the plist matches the modern `IOUSBHostDevice`
  provider class. If the on-connect trigger doesn't fire on your macOS version,
  the periodic `StartInterval` still keeps things in sync; you can also try the
  older `IOUSBDevice` provider class.

## Project layout

```
src/main.rs                      # the tool
dist/com.jasonyuan.ajazz.plist   # LaunchAgent template
dist/install.sh                  # build + install + load the agent
dist/uninstall.sh                # unload + remove the agent
```
