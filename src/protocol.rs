//! What the M913 stores where, and the names this tool uses for it.
//!
//! Memory map (active profile only; the switch under the mouse selects the other one):
//!
//! | addr                | contents                                                        |
//! |---------------------|-----------------------------------------------------------------|
//! | 0x00                | 1000/Hz polling rate, ck                                        |
//! | 0x02                | active DPI level 1-5, ck                                        |
//! | 0x0c                | 5 DPI levels: code, code, 0, ck ([`DPI_CODES`])                 |
//! | 0x2c                | DPI-indicator colours (never written by the vendor tool)        |
//! | 0x54                | lighting R G B ck                                               |
//! | 0x58 / 0x5a / 0x5c  | lighting mode / brightness / speed, each value + ck             |
//! | 0x60                | 16 button records: type p1 p2 ck (order: [`BUTTONS`])           |
//! | 0x100 + 0x20 * rec  | key sequence for record type 05                                 |
//! | 0x300 + 0x180 * n   | macro n (0-14)                                                  |
//!
//! Every record ends in a checksum, `(0x55 - sum of the preceding bytes) & 0xff`.

pub const fn ck(data: &[u8]) -> u8 {
    let mut sum: u8 = 0x55;
    let mut i = 0;
    while i < data.len() {
        sum = sum.wrapping_sub(data[i]);
        i += 1;
    }
    sum
}

pub fn closed(data: &[u8]) -> Vec<u8> {
    let mut v = data.to_vec();
    v.push(ck(data));
    v
}

pub const ADDR_POLLING: u16 = 0x00;
/// 0 or 1: which of the two profiles is active (the button under the mouse toggles them).
pub const ADDR_PROFILE: u16 = 0x04;
pub const ADDR_DPI_ACTIVE: u16 = 0x02;
pub const ADDR_DPI: u16 = 0x0c;
pub const ADDR_COLOR: u16 = 0x54;
pub const ADDR_LIGHT_MODE: u16 = 0x58;
pub const ADDR_BRIGHTNESS: u16 = 0x5a;
pub const ADDR_SPEED: u16 = 0x5c;
pub const ADDR_RECORDS: u16 = 0x60;
pub const ADDR_SLOTS: u16 = 0x100;
pub const SLOT_SIZE: usize = 0x20;
/// Presses and releases one button can play. The slot has room for 10, but the firmware ignores
/// a write of 7 or more (while still echoing it).
pub const SLOT_EVENTS: usize = 6;
pub const ADDR_MACROS: u16 = 0x300;
pub const MACRO_SIZE: usize = 0x180;
pub const MACRO_COUNT: usize = 15;
/// Name area at the start of a macro: length byte + UTF-16LE.
pub const MACRO_NAME: usize = 0x1e;
/// Delay the vendor tool stores after a step that has no explicit wait.
pub const DEFAULT_DELAY: u16 = 3;

/// What the vendor tool writes; backup/restore stay inside these.
pub const BACKUP_RANGES: [(u16, usize); 5] = [
    (0x00, 0x04),
    (0x0c, 0x20),
    (0x54, 0x4c),
    (0x100, 0x200),
    (0x300, MACRO_SIZE * MACRO_COUNT),
];
pub const BACKUP_END: usize = 0x300 + MACRO_SIZE * MACRO_COUNT;

/// Button name -> record index. 1-12 are the thumb grid, numbered as in the vendor tool; fire
/// is the small button beside the left click.
pub const BUTTONS: [(&str, usize); 16] = [
    ("1", 0), ("2", 1), ("3", 2), ("4", 3),
    ("5", 4), ("6", 5), ("7", 8), ("8", 9),
    ("9", 12), ("10", 13), ("11", 14), ("12", 15),
    ("left", 7), ("right", 6), ("middle", 10), ("fire", 11),
];

/// Actions that are a fixed record: name -> (type, p1, p2).
pub const SIMPLE: [(&str, [u8; 3]); 12] = [
    ("disable", [0x00, 0, 0]),
    ("left", [0x01, 0x01, 0]),
    ("right", [0x01, 0x02, 0]),
    ("middle", [0x01, 0x04, 0]),
    ("back", [0x01, 0x08, 0]),
    ("forward", [0x01, 0x10, 0]),
    ("dpi-loop", [0x02, 0x01, 0]),
    ("dpi+", [0x02, 0x02, 0]),
    ("dpi-", [0x02, 0x03, 0]),
    ("three-click", [0x04, 0x32, 0x03]),
    ("polling-rate-switch", [0x07, 0, 0]),
    ("rgb-toggle", [0x08, 0, 0]),
];

pub const T_FIRE: u8 = 0x04;
pub const T_SEQUENCE: u8 = 0x05;
pub const T_MACRO: u8 = 0x06;

/// Event kinds in key sequences and macros; the "up" kind is `kind & 0x0f | 0x40`.
pub const EV_MODIFIER: u8 = 0x80;
pub const EV_KEY: u8 = 0x81;
pub const EV_MEDIA: u8 = 0x82;
pub const EV_MOUSE: u8 = 0x84;

pub const MEDIA: [(&str, u16); 18] = [
    ("media-player", 0x0183), ("play-pause", 0x00cd), ("next", 0x00b5), ("previous", 0x00b6),
    ("stop", 0x00b7), ("mute", 0x00e2), ("volume-up", 0x00e9), ("volume-down", 0x00ea),
    ("email", 0x018a), ("calculator", 0x0192), ("my-computer", 0x0194), ("www-home", 0x0223),
    ("www-search", 0x0221), ("www-forward", 0x0225), ("www-back", 0x0224), ("www-stop", 0x0226),
    ("www-refresh", 0x0227), ("www-favorites", 0x022a),
];

/// Consumer codes Redragon's software doesn't offer, named after what Linux makes of them
/// (drivers/hid/hid-input.c). Any other 16-bit code can be given as a number.
pub const EXTRA_MEDIA: [(&str, u16); 62] = [
    ("brightness-up", 0x006f),
    ("brightness-down", 0x0070),
    ("keyboard-backlight-up", 0x0079),
    ("keyboard-backlight-down", 0x007a),
    ("keyboard-backlight-toggle", 0x007c),
    ("power", 0x0030),
    ("sleep", 0x0032),
    ("menu", 0x0040),
    ("media-play", 0x00b0),
    ("media-pause", 0x00b1),
    ("record", 0x00b2),
    ("fast-forward", 0x00b3),
    ("rewind", 0x00b4),
    ("eject", 0x00b8),
    ("shuffle", 0x00b9),
    ("repeat", 0x00bc),
    ("voice-command", 0x00cf),
    ("dictate", 0x00d8),
    ("emoji-picker", 0x00d9),
    ("bass-boost", 0x00e5),
    ("word-processor", 0x0184),
    ("text-editor", 0x0185),
    ("spreadsheet", 0x0186),
    ("presentation", 0x0188),
    ("calendar", 0x018e),
    ("task-manager", 0x018f),
    ("chat", 0x0199),
    ("log-off", 0x019c),
    ("screen-lock", 0x019e),
    ("control-panel", 0x019f),
    ("app-switcher", 0x01a2),
    ("help", 0x01a6),
    ("documents", 0x01a7),
    ("screensaver", 0x01b1),
    ("file-manager", 0x01b4),
    ("images", 0x01b6),
    ("audio", 0x01b7),
    ("video", 0x01b8),
    ("assistant", 0x01cb),
    ("new", 0x0201),
    ("open", 0x0202),
    ("close", 0x0203),
    ("save", 0x0207),
    ("print-document", 0x0208),
    ("properties", 0x0209),
    ("undo", 0x021a),
    ("copy", 0x021b),
    ("cut", 0x021c),
    ("paste", 0x021d),
    ("find", 0x021f),
    ("zoom-in", 0x022d),
    ("zoom-out", 0x022e),
    ("zoom-reset", 0x022f),
    ("full-screen", 0x0232),
    ("scroll-up", 0x0233),
    ("scroll-down", 0x0234),
    ("redo", 0x0279),
    ("reply", 0x0289),
    ("forward-mail", 0x028b),
    ("send", 0x028c),
    ("keyboard-layout-next", 0x029d),
    ("all-applications", 0x02a2),
];

pub const MODIFIERS: [(&str, u16); 4] = [("ctrl", 0x01), ("shift", 0x02), ("alt", 0x04), ("win", 0x08)];
pub const MOUSE: [(&str, u16); 5] =
    [("left", 0x01), ("right", 0x02), ("middle", 0x04), ("back", 0x08), ("forward", 0x10)];

pub const POLLING: [(u16, u8); 4] = [(125, 0x08), (250, 0x04), (500, 0x02), (1000, 0x01)];
pub const LIGHT_MODES: [&str; 4] = ["off", "steady", "breathing", "neon"];
pub const REPEAT_WHILE_HELD: u8 = 0xfe;
pub const REPEAT_UNTIL_KEY: u8 = 0xff;

/// DPI 100..16000 in steps of 100 -> sensor code, as Redragon's tool writes it. Irregular, so a
/// table rather than a formula.
pub const DPI_CODES: [u8; 160] = [
    0x00, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x10, 0x11, 0x12,
    0x13, 0x14, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x23, 0x25, 0x26,
    0x27, 0x28, 0x29, 0x2a, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x32, 0x33, 0x34, 0x35, 0x36, 0x38, 0x39,
    0x3a, 0x3b, 0x3c, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x44, 0x45, 0x46, 0x47, 0x48, 0x4a, 0x4b, 0x4c,
    0x4d, 0x4e, 0x50, 0x51, 0x52, 0x53, 0x54, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5c, 0x5d, 0x5e, 0x5f,
    0x60, 0x62, 0x63, 0x64, 0x65, 0x66, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6e, 0x6f, 0x70, 0x71, 0x72,
    0x74, 0x75, 0x76, 0x77, 0x78, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x80, 0x81, 0x82, 0x83, 0x84, 0x86,
    0x87, 0x88, 0x89, 0x8a, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x92, 0x93, 0x94, 0x95, 0x96, 0x98, 0x99,
    0x9a, 0x9b, 0x9c, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xaa, 0xab, 0xac,
    0xad, 0xae, 0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd,
];

/// HID keyboard usage for a key name, or None.
pub fn key_code(name: &str) -> Option<u16> {
    let b = name.as_bytes();
    if b.len() == 1 && b[0].is_ascii_lowercase() {
        return Some(0x04 + (b[0] - b'a') as u16);
    }
    if b.len() == 1 && b[0].is_ascii_digit() {
        let d = (b[0] - b'0') as u16;
        return Some(if d == 0 { 0x27 } else { 0x1e + d - 1 });
    }
    if let Some(n) = name.strip_prefix('f').and_then(|n| n.parse::<u16>().ok()) {
        return match n {
            1..=12 => Some(0x3a + n - 1),
            13..=24 => Some(0x68 + n - 13),
            _ => None,
        };
    }
    if let Some(d) = name.strip_prefix("kp").and_then(|n| n.parse::<u16>().ok()) {
        return match d {
            0 => Some(0x62),
            1..=9 => Some(0x59 + d - 1),
            _ => None,
        };
    }
    NAMED_KEYS.iter().find(|(n, _)| *n == name).map(|(_, c)| *c)
}

pub fn key_name(code: u16) -> Option<String> {
    match code {
        0x04..=0x1d => Some(((b'a' + (code - 0x04) as u8) as char).to_string()),
        0x1e..=0x26 => Some((code - 0x1e + 1).to_string()),
        0x27 => Some("0".into()),
        0x3a..=0x45 => Some(format!("f{}", code - 0x3a + 1)),
        0x68..=0x73 => Some(format!("f{}", code - 0x68 + 13)),
        0x59..=0x61 => Some(format!("kp{}", code - 0x59 + 1)),
        0x62 => Some("kp0".into()),
        _ => NAMED_KEYS.iter().find(|(_, c)| *c == code).map(|(n, _)| n.to_string()),
    }
}

const NAMED_KEYS: [(&str, u16); 26] = [
    ("enter", 0x28), ("esc", 0x29), ("backspace", 0x2a), ("tab", 0x2b), ("space", 0x2c),
    ("minus", 0x2d), ("equal", 0x2e), ("capslock", 0x39), ("print", 0x46), ("scrolllock", 0x47),
    ("pause", 0x48), ("insert", 0x49), ("home", 0x4a), ("pageup", 0x4b), ("delete", 0x4c),
    ("end", 0x4d), ("pagedown", 0x4e), ("arrow-right", 0x4f), ("arrow-left", 0x50),
    ("arrow-down", 0x51), ("arrow-up", 0x52), ("kp/", 0x54), ("kp*", 0x55), ("kp-", 0x56),
    ("kp+", 0x57), ("kpenter", 0x58),
];

pub fn lookup<T: Copy>(table: &[(&str, T)], name: &str) -> Option<T> {
    table.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
}

pub fn reverse<T: PartialEq>(table: &[(&'static str, T)], value: T) -> Option<&'static str> {
    table.iter().find(|(_, v)| *v == value).map(|(n, _)| *n)
}
