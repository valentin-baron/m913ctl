# Config reference

An m913ctl config is a YAML (or JSON) file with up to five sections. `m913ctl apply FILE` writes
**only the sections and fields present** — a file with just `polling_rate: 1000` changes nothing
else. `m913ctl get` prints the mouse's current state in this same format, so the quickest start
is:

```
m913ctl get > m913.yaml
```

Every write is read back from the mouse and checked; `m913ctl check FILE` validates a file without
touching the mouse.

For completion and error checking in editors with yaml-language-server, start the file with:

```yaml
# yaml-language-server: $schema=https://raw.githubusercontent.com/valentin-baron/m913ctl/main/m913ctl.schema.yaml
```

- [`polling_rate`](#polling_rate)
- [`dpi`](#dpi)
- [`lighting`](#lighting)
- [`macros`](#macros)
- [`buttons`](#buttons)
- [Key names](#key-names)
- [Limits](#limits)
- [Full example](#full-example)

## `polling_rate`

How often the mouse reports to the computer, in Hz.

```yaml
polling_rate: 1000
```

| value | |
|---|---|
| `125` `250` `500` `1000` | Hz |

## `dpi`

The five DPI levels the DPI buttons step through, and which one is active.

```yaml
dpi:
  levels: [800, 1600, 3200, 6400, 12000]
  active: 2
```

| field | value | |
|---|---|---|
| `levels` | list of exactly 5 | each 100–16000, in steps of 100 |
| `active` | 1–5 | the level in use now |

Either field can be given alone.

## `lighting`

The logo light. It is only visible while the mouse's eco mode is off. The side-button backlight is
always red and can't be changed.

```yaml
lighting:
  mode: breathing
  color: "#00ff80"
  speed: 2
```

| field | value | used by |
|---|---|---|
| `mode` | `off` `steady` `breathing` `neon` | |
| `color` | `"#rrggbb"` — quote it, `#` starts a YAML comment | steady, breathing |
| `brightness` | 1–255 | steady |
| `speed` | 1–5 | breathing, neon |

Each field is written only when given: `lighting: {color: "#ff0000"}` changes the colour and keeps
the mode. Redragon's software offers no colour for `neon`.

## `macros`

Named sequences of key presses, mouse clicks and pauses, which buttons can then play.

```yaml
macros:
  hello: [down shift, down h, up h, up shift, down e, up e, down l, up l, down l, up l, down o, up o]
  burst: [down left, wait 40, up left, wait 40, down left, wait 40, up left]
```

The order matters: the mouse stores macros by position (1–15), and `apply` writes them in the
order of the file. A button refers to a macro by name — in the same file, or (with
`m913ctl set BUTTON macro NAME`) to one already in the mouse; `m913ctl get macros` lists those.

**Steps**

| step | |
|---|---|
| `down X` | press X |
| `up X` | release X |
| `wait MS` | pause after the previous step, in milliseconds (up to 65535) |

`X` is a [key](#key-names), a modifier (`ctrl` `shift` `alt` `win`) or a mouse button (`left`
`right` `middle` `back` `forward`). A step without a `wait` after it is followed by 3 ms, the
same as Redragon's software. A macro can't start with a `wait`.

| limit | |
|---|---|
| macros | 15 |
| name | 14 characters |
| steps | 70 presses and releases per macro (waits don't count) |

How often a macro plays is set where a button uses it, see [`macro`](#macro).

## `buttons`

What each button does.

```yaml
# hello and burst: the macros from the example above
buttons:
  1: {key: a}
  2: {key: ctrl+c}
  3: back
  4: {media: play-pause}
  5: {macro: hello}
  6: {macro: burst, repeat: while-held}
  12: dpi-loop
  fire: {key: f13}
```

| button | |
|---|---|
| `1` … `12` | the thumb grid, numbered as in Redragon's software |
| `left` `right` `middle` | the main buttons and the wheel click |
| `fire` | the small button beside the left button |

The two DPI buttons behind the wheel are fixed in the firmware (DPI up / down) and can't be
remapped. Careful with `left`: a config that takes the left click away leaves you without one
until you change it back.

### Simple actions

Written as a plain word: `3: back`.

| action | |
|---|---|
| `left` `right` `middle` | mouse buttons |
| `back` `forward` | mouse buttons 4 and 5 |
| `disable` | does nothing |
| `dpi+` `dpi-` | next / previous DPI level |
| `dpi-loop` | cycle through the DPI levels |
| `three-click` | triple left click |
| `polling-rate-switch` | cycle the polling rate |
| `rgb-toggle` | lighting on / off |

### `key`

A key, optionally with modifiers joined by `+`. The key is held as long as the button is.

```yaml
buttons:
  1: {key: a}
  2: {key: ctrl+shift+t}
  3: {key: f13}
```

Modifiers: `ctrl` `shift` `alt` `win`. At most two modifiers with one key (see [Limits](#limits)).
Keys Redragon's software doesn't offer, such as `f13`–`f24`, work as well.

### `media`

```yaml
buttons:
  4: {media: volume-up}
```

| | |
|---|---|
| playback | `play-pause` `next` `previous` `stop` |
| volume | `mute` `volume-up` `volume-down` |
| apps | `media-player` `email` `calculator` `my-computer` |
| browser | `www-home` `www-search` `www-back` `www-forward` `www-stop` `www-refresh` `www-favorites` |

### `macro`

Plays a macro.

```yaml
# hello and burst: the macros from the macros example
buttons:
  5: {macro: hello}               # once
  6: {macro: burst, repeat: 3}
  7: {macro: burst, repeat: while-held}
  8: {macro: hello, repeat: until-key}
```

| `repeat` | |
|---|---|
| 1–253 | that many times (default 1) |
| `while-held` | over and over while the button is held |
| `until-key` | over and over until any key is pressed |

### `fire`

Repeated left clicks, Redragon's "fire key".

```yaml
buttons:
  fire: {fire: {speed: 58, times: 3}}
```

| field | |
|---|---|
| `speed` | 0–255, the raw value the mouse stores; lower is faster |
| `times` | 0–255 |

`three-click` is the same thing with Redragon's preset values.

### `experimental`

Things the mouse does that Redragon's software doesn't offer. They work, but behave in ways the
regular actions don't.

#### `key_sequence`

Several keys (or combos) from one press of the button, without a macro.

```yaml
buttons:
  9: {experimental: {key_sequence: [h, i]}}
  10: {experimental: {key_sequence: [ctrl+c, x]}}
```

The mouse plays everything up to the first key release when the button goes down, and the rest
when it comes up. So `[h, i]` holds `h` for as long as the button is held — long enough and the
system's key repeat types `hhhh` — and types `i` on release. The keys after the first come 3
polls apart (23 ms at 125 Hz).

Up to 6 presses and releases in total: 3 plain keys, or e.g. `ctrl+c` plus one key. Use a
[macro](#macros) for anything longer or with exact timing.

## Key names

| | |
|---|---|
| letters | `a` … `z` |
| digits | `0` … `9` |
| function keys | `f1` … `f24` |
| keypad | `kp0` … `kp9` `kp+` `kp-` `kp*` `kp/` `kpenter` |
| editing | `enter` `esc` `backspace` `tab` `space` `delete` `insert` |
| navigation | `home` `end` `pageup` `pagedown` `arrow-up` `arrow-down` `arrow-left` `arrow-right` |
| punctuation | `minus` `equal` |
| other | `capslock` `print` `scrolllock` `pause` |

Names are US-layout key positions, as the mouse sends them: on a German layout, `y` produces `z`.

## Limits

These come from the mouse's firmware, and m913ctl refuses a config that exceeds them rather than
let the mouse silently drop it:

| | |
|---|---|
| keys on one button (`key`, `media`, `key_sequence`) | 6 presses and releases: `ctrl+shift+a` is 6, `ctrl+shift+alt+a` is 8 |
| macros | 15, 14-character names, 70 presses and releases each |
| DPI levels | 5 |

## Full example

```yaml
# yaml-language-server: $schema=https://raw.githubusercontent.com/valentin-baron/m913ctl/main/m913ctl.schema.yaml
polling_rate: 1000
dpi:
  levels: [800, 1600, 3200, 6400, 12000]
  active: 2
lighting: {mode: steady, color: "#ff8000", brightness: 128}
macros:
  hello: [down shift, down h, up h, up shift, down e, up e, down l, up l, down l, up l, down o, up o]
  clicker: [down left, wait 40, up left, wait 300]
buttons:
  1: {key: a}
  2: {key: shift+b}
  3: back
  4: forward
  5: {media: volume-up}
  6: {media: mute}
  7: {macro: hello}
  8: {macro: hello, repeat: 3}
  9: {macro: hello, repeat: until-key}
  10: {macro: clicker, repeat: while-held}
  11: dpi-loop
  12: rgb-toggle
  left: left
  right: right
  middle: middle
  fire: {key: f13}
```
