# Changelog

## Unreleased

- `--wait` (any command): if the mouse is in standby, wait until it's moved instead of failing.
- `m913ctl keepalive on|off` keeps an awake mouse from going into standby, for a GUI that stays open (Redragon's software sets the same flag while it runs). It can't wake a sleeping mouse, and it costs battery while on.

## 0.0.2 - 2026-11-24

- `m913ctl get battery` shows the battery level, in steps of 10% as the mouse reports it, and
  says when the mouse is asleep instead of timing out.
- The README points Arch users to the [AUR package](https://aur.archlinux.org/packages/m913ctl).

## 0.0.1 - 2026-11-24

First release.

- `get` / `set` for single settings, `apply` / `check` for whole YAML or JSON configs, `dump`,
  `backup` / `restore`, shell completions.
- Polling rate, DPI levels, lighting, macros and all 16 buttons, including keys with modifiers,
  media keys and the experimental `key_sequence` and `custom_media_button`.
- Every write is read back from the mouse and checked.
- Linux and Windows builds.
