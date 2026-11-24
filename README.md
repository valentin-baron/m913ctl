# m913ctl

Configure the **Redragon M913 (Impact Elite)** without Redragon's Windows software: button
mapping, macros, DPI levels, lighting and polling rate, from a YAML file. Everything is written
into the mouse itself, so it keeps working on any computer afterwards.

Works on Linux and Windows (and should on macOS, untested), with the wireless receiver
(`25a7:fa07`) or the cable (`25a7:fa08`).

```
m913ctl get                       # everything, in the config format
m913ctl get dpi                   # or: polling-rate, lighting, buttons, button <BUTTON>
m913ctl set fire key f13          # one button
m913ctl set side_3 media play-pause
m913ctl set dpi 2 1600            # one DPI level
m913ctl set lighting --mode steady --color "#ff8000"
m913ctl apply m913.yaml           # a whole config file
```

Every command and value is listed in `--help` at each level (`m913ctl set --help`,
`m913ctl set fire --help`, `m913ctl set fire key --help`), and a mistyped value prints the valid
ones. Shell completions: `m913ctl completions fish > ~/.config/fish/completions/m913ctl.fish`
(also `bash`, `zsh`, `elvish`, `powershell`).

| command | |
|---|---|
| `get [PART]` | current settings, all of them or one part |
| `set …` | change one setting: `polling-rate`, `dpi`, `dpi-levels`, `active-dpi`, `lighting`, or a button (`side_1` … `side_12`, `left`, `right`, `middle`, `fire`) |
| `apply [FILE]` | write a config file; default `$XDG_CONFIG_HOME/m913ctl/config.yaml` |
| `check [FILE]` | validate a config file without touching the mouse |
| `dump` | same as `get` |
| `backup FILE` / `restore FILE` | save / write back the mouse's settings memory |

Macros can only be set through a config file.

## Install

```
cargo install --git https://github.com/valentin-baron/m913ctl
```

**Linux** needs permission to talk to the mouse. Install the udev rule, then replug the receiver
or cable:

```
sudo cp 70-m913.rules /etc/udev/rules.d/
sudo udevadm control --reload
```

**Windows** needs nothing extra.

## Config

YAML or JSON (any JSON file is valid YAML too). For completion and error checking in editors with
yaml-language-server (VS Code, Neovim, …), start the file with:

```yaml
# yaml-language-server: $schema=https://raw.githubusercontent.com/valentin-baron/m913ctl/main/m913ctl.schema.yaml
```

The same schema is in `m913ctl.schema.json` for JSON editors.

```yaml
polling_rate: 1000            # 125 | 250 | 500 | 1000
dpi:
  levels: [800, 1600, 3200, 6400, 12000]   # exactly 5, 100-16000 in steps of 100
  active: 2                   # 1-5
lighting:
  mode: steady                # off | steady | breathing | neon
  color: "#ff8000"            # steady, breathing
  brightness: 128             # 1-255, steady
  speed: 3                    # 1-5, breathing and neon
macros:                       # up to 15; names up to 14 characters
  hello: [down shift, down h, up h, up shift, down e, up e, down l, up l, down l, up l, down o, up o]
  clicker: [down left, wait 40, up left, wait 300]
buttons:
  side_1: {key: a}
  side_2: {key: ctrl+shift+t}
  side_3: back
  side_4: {media: play-pause}
  side_5: {macro: hello, repeat: 1}
  side_6: {macro: clicker, repeat: while-held}
  side_12: dpi-loop
  fire: {key: f13}
```

**Buttons**: `side_1` … `side_12` are the thumb grid, numbered as in Redragon's software; `left`,
`right`, `middle`; `fire` is the small button beside the left click. The two DPI buttons behind
the wheel are fixed in the firmware and can't be remapped.

**Actions**

| action | |
|---|---|
| `left` `right` `middle` `back` `forward` | mouse buttons |
| `disable` | does nothing |
| `dpi+` `dpi-` `dpi-loop` | DPI up, down, cycle |
| `three-click` | triple left click |
| `polling-rate-switch` `rgb-toggle` | cycle the polling rate, lighting on/off |
| `{key: ctrl+shift+a}` | a key, optionally with `ctrl` `shift` `alt` `win` |
| `{media: volume-up}` | `play-pause` `next` `previous` `stop` `mute` `volume-up` `volume-down` `media-player` `email` `calculator` `my-computer` `www-home` `www-search` `www-back` `www-forward` `www-stop` `www-refresh` `www-favorites` |
| `{macro: NAME, repeat: N}` | a macro; `repeat` is 1-253, `while-held`, or `until-key` (until any key is pressed) |
| `{fire: {speed: N, times: N}}` | repeated left clicks; `speed` is the raw stored value, lower is faster |

**Keys**: `a`-`z`, `0`-`9`, `f1`-`f24`, `kp0`-`kp9`, `kp+` `kp-` `kp*` `kp/` `kpenter`, `enter`
`esc` `backspace` `tab` `space` `minus` `equal` `capslock` `print` `scrolllock` `pause` `insert`
`home` `pageup` `delete` `end` `pagedown` `arrow-up` `arrow-down` `arrow-left` `arrow-right`.

**Macro steps**: `down X`, `up X` and `wait MS` (the delay after the step before it; 3 ms when
there is none). `X` is a key, a modifier, or a mouse button.

## Good to know

- **The mouse sleeps.** A wireless M913 that hasn't moved for a while stops answering, and
  m913ctl reports "no answer … mouse asleep?". Move the mouse and run the command again.
- **Lighting** only shows with the mouse's eco mode off. The side-button backlight is always red;
  the firmware doesn't let it change.
- **Profiles**: the switch under the mouse selects one of two profiles; m913ctl reads and writes
  whichever is active.
- **Not supported**: the DPI-indicator colours, which Redragon's software never changes either.

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
