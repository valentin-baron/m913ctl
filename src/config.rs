//! YAML config -> the writes that put it into the mouse.

use serde_yaml_ng::{Mapping, Value};

use crate::protocol::*;

pub struct Write {
    pub addr: u16,
    pub data: Vec<u8>,
    pub what: String,
}

/// Validates the whole config before anything is written, so a typo never leaves the mouse
/// half-configured.
pub fn plan(text: &str) -> Result<Vec<Write>, String> {
    let root: Value = serde_yaml_ng::from_str(text).map_err(|e| e.to_string())?;
    let root = root.as_mapping().ok_or("the config must be a mapping")?;
    let mut writes = Vec::new();
    let mut push = |addr: u16, data: Vec<u8>, what: String| writes.push(Write { addr, data, what });

    for key in root.keys() {
        let k = key.as_str().unwrap_or("");
        if !["polling_rate", "dpi", "lighting", "macros", "buttons"].contains(&k) {
            return Err(format!("unknown section {key:?} (polling_rate, dpi, lighting, macros, buttons)"));
        }
    }

    if let Some(v) = root.get("polling_rate") {
        let hz = v.as_u64().ok_or("polling_rate must be a number")? as u16;
        let code = POLLING.iter().find(|(h, _)| *h == hz).map(|(_, c)| *c)
            .ok_or("polling_rate must be 125, 250, 500 or 1000")?;
        push(ADDR_POLLING, closed(&[code]), format!("polling rate {hz} Hz"));
    }

    if let Some(dpi) = root.get("dpi") {
        let dpi = dpi.as_mapping().ok_or("dpi must be a mapping")?;
        if let Some(levels) = dpi.get("levels") {
            let levels = levels.as_sequence().ok_or("dpi.levels must be a list")?;
            if levels.len() != 5 {
                return Err("dpi.levels must have exactly 5 entries".into());
            }
            let mut data = Vec::new();
            for l in levels {
                let d = l.as_u64().filter(|d| (100..=16000).contains(d) && d % 100 == 0)
                    .ok_or(format!("dpi level {l:?} must be 100-16000 in steps of 100"))?;
                let c = DPI_CODES[(d / 100 - 1) as usize];
                data.extend(closed(&[c, c, 0]));
            }
            push(ADDR_DPI, data, format!("dpi levels {}", show_list(levels)));
        }
        if let Some(a) = dpi.get("active") {
            let a = a.as_u64().filter(|a| (1..=5).contains(a)).ok_or("dpi.active must be 1-5")?;
            push(ADDR_DPI_ACTIVE, closed(&[a as u8]), format!("dpi level {a} active"));
        }
    }

    if let Some(light) = root.get("lighting") {
        let light = light.as_mapping().ok_or("lighting must be a mapping")?;
        // Each field is written only when given, so changing the color keeps the mode.
        let mode = match light.get("mode") {
            Some(Value::Bool(false)) => Some("off".to_string()), // YAML 1.1 reads `off` as false
            Some(v) => Some(v.as_str().ok_or("lighting.mode must be a string")?.to_lowercase()),
            None => None,
        };
        let mode = match mode {
            Some(mode) => Some((
                LIGHT_MODES.iter().position(|x| *x == mode)
                    .ok_or("lighting.mode must be off, steady, breathing or neon")?,
                mode,
            )),
            None => None,
        };
        if let Some(c) = light.get("color") {
            let c = c.as_str().ok_or("lighting.color must be \"#rrggbb\"")?;
            let rgb = parse_color(c).ok_or(format!("bad color {c:?} (want #rrggbb)"))?;
            push(ADDR_COLOR, closed(&rgb), format!("color {c}"));
        }
        if let Some(b) = light.get("brightness") {
            let b = b.as_u64().filter(|b| (1..=255).contains(b)).ok_or("lighting.brightness must be 1-255")?;
            push(ADDR_BRIGHTNESS, closed(&[b as u8]), format!("brightness {b}"));
        }
        if let Some(s) = light.get("speed") {
            let s = s.as_u64().filter(|s| (1..=5).contains(s)).ok_or("lighting.speed must be 1-5")?;
            push(ADDR_SPEED, closed(&[s as u8]), format!("lighting speed {s}"));
        }
        if let Some((m, mode)) = mode {
            push(ADDR_LIGHT_MODE, closed(&[m as u8]), format!("lighting {mode}"));
        }
    }

    let empty = Mapping::new();
    let macros = match root.get("macros") {
        Some(m) => m.as_mapping().ok_or("macros must be a mapping of name -> steps")?,
        None => &empty,
    };
    if macros.len() > MACRO_COUNT {
        return Err(format!("at most {MACRO_COUNT} macros"));
    }
    let mut macro_names = Vec::new();
    for (i, (name, steps)) in macros.iter().enumerate() {
        let name = scalar(name).ok_or("macro names must be strings")?;
        let steps = steps.as_sequence().ok_or(format!("macro {name:?} must be a list of steps"))?;
        let data = encode_macro(&name, steps)?;
        push(ADDR_MACROS + (MACRO_SIZE * i) as u16, data, format!("macro {} {name:?}", i + 1));
        macro_names.push(name);
    }

    if let Some(buttons) = root.get("buttons") {
        let buttons = buttons.as_mapping().ok_or("buttons must be a mapping")?;
        for (name, action) in buttons {
            let name = name.as_str().ok_or("button names must be strings")?;
            let rec = lookup(&BUTTONS, name)
                .ok_or(format!("unknown button {name:?} (side_1..side_12, left, right, middle, fire)"))?;
            let (record, slot) = encode_button(action, &macro_names)
                .map_err(|e| format!("button {name}: {e}"))?;
            if let Some(slot) = slot {
                push(ADDR_SLOTS + (SLOT_SIZE * rec) as u16, slot, String::new());
            }
            push(ADDR_RECORDS + 4 * rec as u16, record, format!("{name}: {}", show(action)));
        }
    }

    Ok(writes)
}

fn encode_button(action: &Value, macros: &[String]) -> Result<(Vec<u8>, Option<Vec<u8>>), String> {
    if let Some(s) = action.as_str() {
        let rec = lookup(&SIMPLE, s).ok_or(format!("unknown action {s:?}"))?;
        return Ok((closed(&rec), None));
    }
    let map = action.as_mapping().ok_or(format!("bad action {}", show(action)))?;
    let events: Vec<(u8, u16)> = if let Some(k) = map.get("key") {
        parse_combo(&scalar(k).ok_or("key must be a string")?)?
    } else if let Some(m) = map.get("media") {
        let m = m.as_str().ok_or("media must be a string")?;
        let u = lookup(&MEDIA, m).ok_or(format!("unknown media key {m:?}"))?;
        vec![(EV_MEDIA, u), (EV_MEDIA & 0x0f | 0x40, u)]
    } else if let Some(name) = map.get("macro") {
        let name = scalar(name).ok_or("macro must be a name")?;
        let index = macros.iter().position(|m| *m == name)
            .ok_or(format!("macro {name:?} is not defined under macros"))?;
        let repeat = match map.get("repeat") {
            None => 1,
            Some(Value::String(s)) if s == "while-held" => REPEAT_WHILE_HELD,
            Some(Value::String(s)) if s == "until-key" => REPEAT_UNTIL_KEY,
            Some(v) => v.as_u64().filter(|n| (1..=253).contains(n))
                .ok_or("repeat must be 1-253, while-held or until-key")? as u8,
        };
        return Ok((closed(&[T_MACRO, index as u8, repeat]), None));
    } else if let Some(f) = map.get("fire") {
        // Repeated left clicks; speed is the raw stored byte (lower is faster).
        let speed = f.get("speed").and_then(Value::as_u64).filter(|s| *s <= 255).ok_or("fire.speed must be 0-255")?;
        let times = f.get("times").and_then(Value::as_u64).filter(|t| *t <= 255).ok_or("fire.times must be 0-255")?;
        return Ok((closed(&[T_FIRE, speed as u8, times as u8]), None));
    } else {
        return Err(format!("bad action {}", show(action)));
    };

    let mut body = vec![events.len() as u8];
    for (kind, code) in &events {
        body.extend([*kind, *code as u8, (*code >> 8) as u8]);
    }
    if body.len() + 1 > SLOT_SIZE {
        return Err("too many keys for one button".into());
    }
    Ok((closed(&[T_SEQUENCE, 0, 0]), Some(closed(&body))))
}

/// "ctrl+shift+a" -> modifier downs, key down, then the ups in the same order.
fn parse_combo(text: &str) -> Result<Vec<(u8, u16)>, String> {
    let text = text.to_lowercase();
    let mut parts: Vec<String> = text.split('+').map(|p| p.trim().to_string()).collect();
    // "kp+" splits into "kp" and ""; glue it back.
    if parts.len() > 1 && parts.last().unwrap().is_empty() {
        parts.pop();
        let last = parts.pop().unwrap();
        parts.push(last + "+");
    }
    let key = parts.pop().unwrap();
    let code = key_code(&key).ok_or(format!("unknown key {key:?} in {text:?}"))?;
    let mut down = Vec::new();
    for m in &parts {
        down.push((EV_MODIFIER, lookup(&MODIFIERS, m).ok_or(format!("unknown modifier {m:?} in {text:?}"))?));
    }
    down.push((EV_KEY, code));
    let up: Vec<_> = down.iter().map(|(k, c)| (k & 0x0f | 0x40, *c)).collect();
    Ok(down.into_iter().chain(up).collect())
}

/// Steps are "down X", "up X" and "wait MS"; a wait is the delay after the step before it.
fn encode_macro(name: &str, steps: &[Value]) -> Result<Vec<u8>, String> {
    let mut events: Vec<(u8, u16, u32)> = Vec::new();
    for step in steps {
        let step = scalar(step).ok_or(format!("macro {name:?}: steps must be strings"))?.to_lowercase();
        let (verb, arg) = step.split_once(' ').map(|(v, a)| (v, a.trim())).unwrap_or((&step, ""));
        match verb {
            "wait" => {
                let ms: u32 = arg.parse().map_err(|_| format!("macro {name:?}: bad wait {arg:?}"))?;
                let last = events.last_mut().ok_or(format!("macro {name:?} starts with a wait"))?;
                last.2 += ms;
            }
            "down" | "up" => {
                let (kind, code) = if let Some(c) = lookup(&MOUSE, arg) {
                    (EV_MOUSE, c)
                } else if let Some(c) = lookup(&MODIFIERS, arg) {
                    (EV_MODIFIER, c)
                } else if let Some(c) = key_code(arg) {
                    (EV_KEY, c)
                } else {
                    return Err(format!("macro {name:?}: unknown key {arg:?}"));
                };
                let kind = if verb == "up" { kind & 0x0f | 0x40 } else { kind };
                events.push((kind, code, 0));
            }
            _ => return Err(format!("macro {name:?}: bad step {step:?} (down X, up X, wait MS)")),
        }
    }

    let raw_name: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
    if raw_name.len() > MACRO_NAME - 1 {
        return Err(format!("macro name {name:?} is too long (14 characters)"));
    }
    let mut head = vec![raw_name.len() as u8];
    head.extend(raw_name);
    head.resize(MACRO_NAME, 0);

    let mut body = (events.len() as u16).to_be_bytes().to_vec();
    for (kind, code, delay) in events {
        let delay = if delay == 0 { DEFAULT_DELAY as u32 } else { delay };
        let delay = u16::try_from(delay).map_err(|_| format!("macro {name:?}: waits are at most 65535 ms"))?;
        body.extend([kind, code as u8, (code >> 8) as u8]);
        body.extend(delay.to_be_bytes());
    }
    if MACRO_NAME + body.len() + 1 > MACRO_SIZE {
        return Err(format!("macro {name:?} has too many steps (70 at most)"));
    }
    head.extend(closed(&body));
    Ok(head)
}

fn parse_color(c: &str) -> Option<[u8; 3]> {
    let h = c.strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(h, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

/// Strings and numbers both, so `key: 1` works as well as `key: "1"`.
fn scalar(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn show(v: &Value) -> String {
    serde_yaml_ng::to_string(v).unwrap_or_default().trim().replace('\n', ", ")
}

fn show_list(v: &[Value]) -> String {
    v.iter().map(show).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The macro the vendor tool wrote during capture, byte for byte.
    #[test]
    fn macro_matches_capture() {
        let steps: Vec<Value> = [
            "down left", "wait 50", "up left", "down right", "wait 50", "up right",
            "down middle", "wait 50", "up middle", "down forward", "wait 50", "up forward",
            "down back", "wait 50", "up back", "down b", "down shift", "wait 1234", "up b", "up shift",
        ].iter().map(|s| Value::String(s.to_string())).collect();
        let mut want = vec![0x02, 0x61];
        want.resize(MACRO_NAME, 0);
        want.extend([
            0x00, 0x0e, 0x84, 0x01, 0x00, 0x00, 0x32, 0x44, 0x01, 0x00, 0x00, 0x03, 0x84, 0x02, 0x00,
            0x00, 0x32, 0x44, 0x02, 0x00, 0x00, 0x03, 0x84, 0x04, 0x00, 0x00, 0x32, 0x44, 0x04, 0x00,
            0x00, 0x03, 0x84, 0x10, 0x00, 0x00, 0x32, 0x44, 0x10, 0x00, 0x00, 0x03, 0x84, 0x08, 0x00,
            0x00, 0x32, 0x44, 0x08, 0x00, 0x00, 0x03, 0x81, 0x05, 0x00, 0x00, 0x03, 0x80, 0x02, 0x00,
            0x04, 0xd2, 0x41, 0x05, 0x00, 0x00, 0x03, 0x40, 0x02, 0x00, 0x00, 0x03, 0xa9,
        ]);
        assert_eq!(encode_macro("a", &steps).unwrap(), want);
    }

    /// Ctrl+Shift+A as captured.
    #[test]
    fn combo_matches_capture() {
        let (rec, slot) = encode_button(&Value::from_mapping("key", "ctrl+shift+a"), &[]).unwrap();
        assert_eq!(rec, [0x05, 0x00, 0x00, 0x50]);
        assert_eq!(slot.unwrap(), [
            0x06, 0x80, 0x01, 0x00, 0x80, 0x02, 0x00, 0x81, 0x04, 0x00,
            0x40, 0x01, 0x00, 0x40, 0x02, 0x00, 0x41, 0x04, 0x00, 0xff,
        ]);
    }

    #[test]
    fn media_matches_capture() {
        let (_, slot) = encode_button(&Value::from_mapping("media", "play-pause"), &[]).unwrap();
        assert_eq!(slot.unwrap(), [0x02, 0x82, 0xcd, 0x00, 0x42, 0xcd, 0x00, 0xf5]);
    }

    #[test]
    fn records_match_capture() {
        let rec = |s: &str| encode_button(&Value::String(s.into()), &[]).unwrap().0;
        assert_eq!(rec("forward"), [0x01, 0x10, 0x00, 0x44]);
        assert_eq!(rec("dpi-"), [0x02, 0x03, 0x00, 0x50]);
        assert_eq!(rec("disable"), [0x00, 0x00, 0x00, 0x55]);
    }

    trait FromMapping {
        fn from_mapping(k: &str, v: &str) -> Value;
    }
    impl FromMapping for Value {
        fn from_mapping(k: &str, v: &str) -> Value {
            let mut m = Mapping::new();
            m.insert(Value::String(k.into()), Value::String(v.into()));
            Value::Mapping(m)
        }
    }
}

#[cfg(test)]
mod schema_tests {
    use serde_yaml_ng::Value;

    /// The JSON schema is a copy of the YAML one; only its $id differs.
    #[test]
    fn schemas_agree() {
        let load = |s: &str| -> Value { serde_yaml_ng::from_str(s).unwrap() };
        let mut yaml = load(include_str!("../m913ctl.schema.yaml"));
        let mut json = load(include_str!("../m913ctl.schema.json"));
        yaml.as_mapping_mut().unwrap().remove("$id");
        json.as_mapping_mut().unwrap().remove("$id");
        assert_eq!(yaml, json);
    }

    /// JSON configs go through the same parser.
    #[test]
    fn json_and_yaml_examples_agree() {
        let yaml = super::plan(include_str!("../examples/full-test.yaml")).unwrap();
        let json = super::plan(include_str!("../examples/full-test.json")).unwrap();
        let bytes = |w: &[super::Write]| w.iter().map(|w| (w.addr, w.data.clone())).collect::<Vec<_>>();
        assert_eq!(bytes(&yaml), bytes(&json));
    }
}
