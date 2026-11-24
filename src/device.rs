//! HID transport via hidapi (Linux hidraw, Windows HID, macOS IOHIDManager): feature report
//! 0x08 out, input report 0x09 back.
//!
//! Both reports live in vendor collections of the same USB interface: 0x08 under usage page
//! 0xff02, 0x09 under 0xff01. Linux exposes the interface as one hidraw node; Windows and macOS
//! open each top-level collection separately, so the request and its answer can travel through
//! two different handles.

use hidapi::{HidApi, HidDevice};

use crate::protocol::closed;

const VENDOR: u16 = 0x25a7;
const PRODUCTS: [u16; 2] = [0xfa07, 0xfa08]; // wireless receiver, cable
const PAGE_REQUEST: u16 = 0xff02;
const PAGE_ANSWER: u16 = 0xff01;
const CHUNK: usize = 10;

pub struct Mouse {
    request: HidDevice,
    /// None when the answers arrive on the request handle (Linux).
    answer: Option<HidDevice>,
}

impl Mouse {
    pub fn open() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| e.to_string())?;
        let ours = |page: u16| {
            api.device_list().find(|d| {
                d.vendor_id() == VENDOR && PRODUCTS.contains(&d.product_id()) && d.usage_page() == page
            })
        };
        let req = ours(PAGE_REQUEST).ok_or("no M913 found (receiver 25a7:fa07 or cable 25a7:fa08)")?;
        let ans = ours(PAGE_ANSWER).unwrap_or(req);
        let open = |info: &hidapi::DeviceInfo| {
            info.open_device(&api).map_err(|e| {
                format!("opening {}: {e} (on Linux: is the udev rule installed and the mouse replugged?)",
                    info.path().to_string_lossy())
            })
        };
        let request = open(req)?;
        let answer = if ans.path() == req.path() { None } else { Some(open(ans)?) };
        Ok(Mouse { request, answer })
    }

    /// The receiver occasionally drops an answer; every request is idempotent, so resend.
    fn request(&mut self, cmd: u8, addr: u16, data: &[u8], len: u8) -> Result<Vec<u8>, String> {
        let mut last = String::new();
        for _ in 0..3 {
            match self.try_request(cmd, addr, data, len) {
                Ok(v) => return Ok(v),
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    fn try_request(&mut self, cmd: u8, addr: u16, data: &[u8], len: u8) -> Result<Vec<u8>, String> {
        let mut msg = vec![0x08, cmd, 0, (addr >> 8) as u8, addr as u8, len];
        msg.extend_from_slice(data);
        msg.resize(16, 0);
        let msg = closed(&msg);
        self.request.send_feature_report(&msg).map_err(|e| format!("sending {}: {e}", hex(&msg)))?;

        // Keyboard reports can share the handle; wait for the echo of our request.
        let header = [0x09, cmd, 0, (addr >> 8) as u8, addr as u8, len];
        let answer = self.answer.as_ref().unwrap_or(&self.request);
        let mut buf = [0u8; 64];
        loop {
            let n = answer.read_timeout(&mut buf, 500).map_err(|e| e.to_string())?;
            if n == 0 {
                return Err(format!("no answer to {} (mouse asleep? move it and retry)", hex(&msg)));
            }
            if n >= 6 + len as usize && buf[..6] == header {
                return Ok(buf[6..6 + len as usize].to_vec());
            }
        }
    }

    /// A status query outside the memory map (online, battery): `08 cmd`, answered by
    /// `09 cmd 00 00 00 len <data>`. Byte 2 of the answer is 0 on success.
    pub fn query(&mut self, cmd: u8) -> Result<Vec<u8>, String> {
        let mut last = String::new();
        for _ in 0..3 {
            match self.try_query(cmd) {
                Ok(v) => return Ok(v),
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    fn try_query(&mut self, cmd: u8) -> Result<Vec<u8>, String> {
        let mut msg = vec![0x08, cmd];
        msg.resize(16, 0);
        let msg = closed(&msg);
        self.request.send_feature_report(&msg).map_err(|e| format!("sending {}: {e}", hex(&msg)))?;

        let answer = self.answer.as_ref().unwrap_or(&self.request);
        let mut buf = [0u8; 64];
        loop {
            let n = answer.read_timeout(&mut buf, 500).map_err(|e| e.to_string())?;
            if n == 0 {
                return Err(format!("no answer to {} (mouse asleep? move it and retry)", hex(&msg)));
            }
            if n >= 6 && buf[0] == 0x09 && buf[1] == cmd {
                if buf[2] != 0 {
                    return Err(format!("the mouse refused {}: {}", hex(&msg), hex(&buf[..n])));
                }
                let len = (buf[5] as usize).min(n - 6);
                return Ok(buf[6..6 + len].to_vec());
            }
        }
    }

    pub fn read(&mut self, addr: u16, length: usize) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(length);
        while out.len() < length {
            let n = CHUNK.min(length - out.len());
            out.extend(self.request(0x08, addr + out.len() as u16, &[], n as u8)?);
        }
        Ok(out)
    }

    /// Writes, then reads back: the mouse echoes writes it silently ignores (an over-long key
    /// sequence, for one), so the echo alone proves nothing.
    pub fn write(&mut self, addr: u16, data: &[u8]) -> Result<(), String> {
        for (i, chunk) in data.chunks(CHUNK).enumerate() {
            self.request(0x07, addr + (i * CHUNK) as u16, chunk, chunk.len() as u8)?;
        }
        let stored = self.read(addr, data.len())?;
        if stored != data {
            return Err(format!(
                "the mouse did not store the write at 0x{addr:04x} (sent {}, holds {})",
                hex(data), hex(&stored)
            ));
        }
        Ok(())
    }
}

pub fn hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
}
