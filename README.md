# m913ctl

Configure the **Redragon M913 (Impact Elite)** without Redragon's Windows software: button
mapping, macros, DPI levels, lighting and polling rate, from a YAML file. Everything is written
into the mouse itself, so it keeps working on any computer afterwards.

Works on Linux and Windows (and should on macOS, untested), with the wireless receiver
(`25a7:fa07`) or the cable (`25a7:fa08`).

```
m913ctl get                         # everything, in the config format
m913ctl get dpi                     # one section: polling-rate, dpi, lighting, buttons
m913ctl get dpi levels              # one value: dpi levels|active|1-5, lighting mode|color|brightness|speed
m913ctl get fire                    # one button: 1-12, left, right, middle, fire
m913ctl get macros                  # the macros stored in the mouse
m913ctl get battery                 # in steps of 10%; also says when the mouse is asleep

m913ctl set fire key f13
m913ctl set 3 media play-pause
m913ctl set dpi 2 1600              # or: set dpi levels 800 1600 3200 6400 12000, set dpi active 2
m913ctl set lighting color "#ff8000"
m913ctl set polling-rate 1000
m913ctl set 5 macro hello --repeat 3  # a macro already in the mouse

m913ctl apply m913.yaml             # a whole config file
```

A command run without its required arguments prints its help, so `m913ctl set fire media` lists
the media keys. Every command and value is listed in `--help` at each level (`m913ctl set --help`,
`m913ctl set fire --help`, `m913ctl set fire key --help`), and a mistyped value prints the valid
ones. Shell completions: `m913ctl completions fish > ~/.config/fish/completions/m913ctl.fish`
(also `bash`, `zsh`, `elvish`, `powershell`).

| command | |
|---|---|
| `get [PART]` | current settings, all of them or one part |
| `set …` | change one setting: `polling-rate`, `dpi …`, `lighting …`, or a button (`1` … `12`, `left`, `right`, `middle`, `fire`) |
| `apply [FILE]` | write a config file; default `$XDG_CONFIG_HOME/m913ctl/config.yaml` |
| `check [FILE]` | validate a config file without touching the mouse |
| `dump` | same as `get` |
| `backup FILE` / `restore FILE` | save / write back the mouse's settings memory |

New macros are defined in a config file (`macros:`) and written with `apply`; after that, `set`
can point any button at them by name.

## Install

**Arch Linux**: from the [AUR](https://aur.archlinux.org/packages/m913ctl), which also installs
the udev rule and shell completions:

```
paru -S m913ctl
```

**Elsewhere**, or for the latest commit:

```
cargo install --git https://github.com/valentin-baron/m913ctl
```

**Linux** (outside the AUR package) needs permission to talk to the mouse. Install the udev rule,
then replug the receiver or cable:

```
sudo cp 70-m913.rules /etc/udev/rules.d/
sudo udevadm control --reload
```

**Windows** needs nothing extra.

## Config

A config file sets polling rate, DPI levels, lighting, macros and what each button does. YAML or
JSON; only what's in the file is written.

```yaml
# yaml-language-server: $schema=https://raw.githubusercontent.com/valentin-baron/m913ctl/main/m913ctl.schema.yaml
dpi: {levels: [800, 1600, 3200, 6400, 12000], active: 2}
lighting: {mode: steady, color: "#ff8000"}
macros:
  hello: [down shift, down h, up h, up shift, down i, up i]
buttons:
  3: back
  4: {media: play-pause}
  5: {macro: hello}
  fire: {key: f13}
```

**[CONFIG.md](CONFIG.md) documents every field, action, key name and limit.** The schema
(`m913ctl.schema.yaml`, also as `.json`) gives editors completion and error checking.

## Good to know

- **The mouse sleeps.** A wireless M913 that hasn't moved for a while stops answering, and
  m913ctl reports "no answer … mouse asleep?". Move the mouse and run the command again.
- **Lighting** only shows with the mouse's eco mode off. The side-button backlight is always red;
  the firmware doesn't let it change.
- **Profiles**: the button on the underside switches between two profiles (it's not a reset
  button). m913ctl reads and writes the active one; `m913ctl get profile` says which.
- **Not supported**: the DPI-indicator colours, which Redragon's software never changes either.
- **Every write is read back and checked.** The mouse acknowledges some writes it then ignores
  (too many keys on one button), so m913ctl doesn't trust the acknowledgement alone.

## How it works

The protocol was captured with usbmon from Redragon's Windows software. Every request is a
16-byte feature report (ID `0x08`), answered by an input report (ID `0x09`) echoing it; the last
byte of both is a checksum, `(0x55 - sum of the other bytes) & 0xff`:

```
08 08 00 HI LO N            read N (<=10) bytes   -> 09 08 00 HI LO N <data>
08 07 00 HI LO N <data>     write N (<=10) bytes  -> echoed
```

The settings are a small memory map; `src/protocol.rs` documents it.

## License

MIT
