//! The mouse's current settings, printed in the config format.

use std::fmt::Write as _;

use crate::device::Mouse;
use crate::protocol::*;

pub fn dump(mouse: &mut Mouse) -> Result<String, String> {
    let mem = mouse.read(0, 0xa0)?;
    let mut out = String::new();

    let polling = POLLING.iter().find(|(_, c)| *c == mem[0]).map(|(h, _)| h.to_string())
        .unwrap_or_else(|| format!("unknown 0x{:02x}", mem[0]));
    writeln!(out, "polling_rate: {polling}").unwrap();

    let levels: Vec<String> = (0..5)
        .map(|i| mem[ADDR_DPI as usize + 4 * i])
        .map(|c| match DPI_CODES.iter().position(|x| *x == c) {
            Some(p) => ((p + 1) * 100).to_string(),
            None => format!("unknown 0x{c:02x}"),
        })
        .collect();
    writeln!(out, "dpi:\n  levels: [{}]\n  active: {}", levels.join(", "), mem[ADDR_DPI_ACTIVE as usize]).unwrap();

    let mode = LIGHT_MODES.get(mem[ADDR_LIGHT_MODE as usize] as usize).copied().unwrap_or("off");
    let rgb = &mem[ADDR_COLOR as usize..ADDR_COLOR as usize + 3];
    let mut light = vec![format!("mode: \"{mode}\"")];
    if matches!(mode, "steady" | "breathing") {
        light.push(format!("color: \"#{:02x}{:02x}{:02x}\"", rgb[0], rgb[1], rgb[2]));
    }
    if mode == "steady" {
        light.push(format!("brightness: {}", mem[ADDR_BRIGHTNESS as usize]));
    }
    if matches!(mode, "breathing" | "neon") {
        light.push(format!("speed: {}", mem[ADDR_SPEED as usize]));
    }
    writeln!(out, "lighting: {{{}}}", light.join(", ")).unwrap();

    let record = |r: usize| &mem[ADDR_RECORDS as usize + 4 * r..ADDR_RECORDS as usize + 4 * r + 4];

    // Only macros some button uses; the rest of macro memory is whatever was left there.
    let mut used: Vec<u8> = BUTTONS.iter().map(|(_, r)| record(*r)).filter(|r| r[0] == T_MACRO).map(|r| r[1]).collect();
    used.sort();
    used.dedup();
    let mut macro_names = Vec::new();
    if !used.is_empty() {
        out.push_str("macros:\n");
    }
    for &i in &used {
        let (name, steps) = read_macro(mouse, i as usize)?;
        writeln!(out, "  {}: [{}]", quote(&name), steps.join(", ")).unwrap();
        macro_names.push((i, name));
    }

    out.push_str("buttons:\n");
    for (name, r) in BUTTONS {
        let rec = record(r);
        let action = match rec[0] {
            T_SEQUENCE => decode_sequence(&mouse.read(ADDR_SLOTS + (SLOT_SIZE * r) as u16, SLOT_SIZE)?),
            T_MACRO => {
                let m = macro_names.iter().find(|(i, _)| *i == rec[1]).map(|(_, n)| quote(n)).unwrap_or_default();
                let repeat = match rec[2] {
                    REPEAT_WHILE_HELD => "while-held".to_string(),
                    REPEAT_UNTIL_KEY => "until-key".to_string(),
                    n => n.to_string(),
                };
                format!("{{macro: {m}, repeat: {repeat}}}")
            }
            _ => match SIMPLE.iter().find(|(_, v)| *v == rec[..3]) {
                Some((n, _)) => n.to_string(),
                None if rec[0] == T_FIRE => format!("{{fire: {{speed: {}, times: {}}}}}", rec[1], rec[2]),
                None => format!("unknown {:02x} {:02x} {:02x}", rec[0], rec[1], rec[2]),
            },
        };
        writeln!(out, "  {name}: {action}").unwrap();
    }
    Ok(out)
}

/// The five DPI levels, for changing one of them.
pub fn dpi_levels(mouse: &mut Mouse) -> Result<Vec<u16>, String> {
    let mem = mouse.read(ADDR_DPI, 20)?;
    (0..5)
        .map(|i| match DPI_CODES.iter().position(|c| *c == mem[4 * i]) {
            Some(p) => Ok((p as u16 + 1) * 100),
            None => Err(format!("DPI level {} holds an unknown code 0x{:02x}; set all five with dpi-levels", i + 1, mem[4 * i])),
        })
        .collect()
}

fn decode_sequence(slot: &[u8]) -> String {
    let n = (slot[0] as usize).min((SLOT_SIZE - 2) / 3);
    let downs: Vec<(u8, u16)> = (0..n)
        .map(|i| (slot[1 + 3 * i], slot[2 + 3 * i] as u16 | (slot[3 + 3 * i] as u16) << 8))
        .filter(|(k, _)| k & 0x80 != 0)
        .collect();
    if let [(EV_MEDIA, u)] = downs[..] {
        return match reverse(&MEDIA, u) {
            Some(m) => format!("{{media: {m}}}"),
            None => format!("{{media: unknown 0x{u:04x}}}"),
        };
    }
    let parts: Vec<String> = downs
        .iter()
        .map(|(k, c)| match *k {
            EV_MODIFIER => reverse(&MODIFIERS, *c).map(str::to_string),
            EV_KEY => key_name(*c),
            _ => None,
        }.unwrap_or_else(|| format!("0x{c:x}")))
        .collect();
    format!("{{key: {}}}", quote(&parts.join("+")))
}

fn read_macro(mouse: &mut Mouse, index: usize) -> Result<(String, Vec<String>), String> {
    let addr = ADDR_MACROS + (MACRO_SIZE * index) as u16;
    let head = mouse.read(addr, MACRO_NAME + 2)?;
    let len = (head[0] as usize).min(MACRO_NAME - 1) & !1;
    let units: Vec<u16> = head[1..1 + len].chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    let mut name = String::from_utf16_lossy(&units);
    if name.is_empty() {
        name = format!("macro{}", index + 1);
    }
    let count = (u16::from_be_bytes([head[MACRO_NAME], head[MACRO_NAME + 1]]) as usize)
        .min((MACRO_SIZE - MACRO_NAME - 3) / 5);
    let raw = mouse.read(addr + MACRO_NAME as u16 + 2, 5 * count)?;
    let mut steps = Vec::new();
    for e in raw.chunks(5) {
        let (kind, code, delay) = (e[0], e[1] as u16 | (e[2] as u16) << 8, u16::from_be_bytes([e[3], e[4]]));
        let what = match kind & 0x0f | 0x80 {
            EV_MOUSE => reverse(&MOUSE, code).map(str::to_string),
            EV_MODIFIER => reverse(&MODIFIERS, code).map(str::to_string),
            EV_KEY => key_name(code),
            _ => None,
        }.unwrap_or_else(|| format!("0x{code:x}"));
        steps.push(quote(&format!("{} {what}", if kind & 0x80 != 0 { "down" } else { "up" })));
        if delay != DEFAULT_DELAY {
            steps.push(format!("wait {delay}"));
        }
    }
    Ok((name, steps))
}

/// Quote anything YAML would read as something other than the plain string.
fn quote(s: &str) -> String {
    let plain = !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || " -_".contains(c))
        && s.parse::<f64>().is_err()
        && !["y", "n", "yes", "no", "on", "off", "true", "false", "null", "~"].contains(&s.to_lowercase().as_str());
    if plain { s.to_string() } else { format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")) }
}
