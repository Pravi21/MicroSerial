use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[allow(non_camel_case_types, non_upper_case_globals, dead_code)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use bindings::*;

#[link(name = "microserial_core", kind = "static")]
unsafe extern "C" {}

pub struct SerialPort {
    handle: *mut ms_serial_port,
    callbacks: Option<Arc<CallbackState>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SerialDevice {
    pub path: String,
    pub description: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, Display)]
pub enum Parity {
    #[strum(to_string = "None")]
    None,
    #[strum(to_string = "Even")]
    Even,
    #[strum(to_string = "Odd")]
    Odd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, Display)]
pub enum StopBits {
    #[strum(to_string = "1")]
    One,
    #[strum(to_string = "2")]
    Two,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, Display)]
pub enum FlowControl {
    #[strum(to_string = "None")]
    None,
    #[strum(to_string = "RTS/CTS")]
    RtsCts,
    #[strum(to_string = "XON/XOFF")]
    XonXoff,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerialConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub rx_buffer_size: u32,
    pub tx_buffer_size: u32,
    pub read_timeout_ms: u32,
    pub write_timeout_ms: u32,
}

impl SerialConfig {
    pub fn to_raw(&self) -> ms_serial_config {
        #[allow(non_upper_case_globals)]
        ms_serial_config {
            baud_rate: self.baud_rate,
            data_bits: self.data_bits,
            stop_bits: match self.stop_bits {
                StopBits::One => 1,
                StopBits::Two => 2,
            },
            parity: match self.parity {
                Parity::None => ms_serial_parity_MS_SERIAL_PARITY_NONE,
                Parity::Even => ms_serial_parity_MS_SERIAL_PARITY_EVEN,
                Parity::Odd => ms_serial_parity_MS_SERIAL_PARITY_ODD,
            },
            flow_control: match self.flow_control {
                FlowControl::None => ms_serial_flow_control_MS_SERIAL_FLOW_NONE,
                FlowControl::RtsCts => ms_serial_flow_control_MS_SERIAL_FLOW_RTS_CTS,
                FlowControl::XonXoff => ms_serial_flow_control_MS_SERIAL_FLOW_XON_XOFF,
            },
            rx_buffer_size: self.rx_buffer_size,
            tx_buffer_size: self.tx_buffer_size,
            read_timeout_ms: self.read_timeout_ms,
            write_timeout_ms: self.write_timeout_ms,
        }
    }

    #[allow(non_upper_case_globals)]
    pub fn from_raw(raw: ms_serial_config) -> Self {
        Self {
            baud_rate: raw.baud_rate,
            data_bits: raw.data_bits as u8,
            stop_bits: match raw.stop_bits {
                2 => StopBits::Two,
                _ => StopBits::One,
            },
            parity: match raw.parity {
                ms_serial_parity_MS_SERIAL_PARITY_EVEN => Parity::Even,
                ms_serial_parity_MS_SERIAL_PARITY_ODD => Parity::Odd,
                _ => Parity::None,
            },
            flow_control: match raw.flow_control {
                ms_serial_flow_control_MS_SERIAL_FLOW_RTS_CTS => FlowControl::RtsCts,
                ms_serial_flow_control_MS_SERIAL_FLOW_XON_XOFF => FlowControl::XonXoff,
                _ => FlowControl::None,
            },
            rx_buffer_size: raw.rx_buffer_size,
            tx_buffer_size: raw.tx_buffer_size,
            read_timeout_ms: raw.read_timeout_ms,
            write_timeout_ms: raw.write_timeout_ms,
        }
    }
}

impl Default for SerialConfig {
    fn default() -> Self {
        SerialConfig::from_raw(default_config_raw())
    }
}

struct CallbackState {
    on_data: Mutex<Box<dyn FnMut(&[u8]) + Send + 'static>>,
    on_event: Mutex<Box<dyn FnMut(i32, &str) + Send + 'static>>,
}

unsafe extern "C" fn data_trampoline(data: *const u8, length: usize, user_data: *mut c_void) {
    if data.is_null() || user_data.is_null() {
        return;
    }
    let slice = unsafe { slice::from_raw_parts(data, length) };
    let state = unsafe { &*(user_data as *const CallbackState) };
    if let Ok(mut guard) = state.on_data.lock() {
        (guard.as_mut())(slice);
    }
}

unsafe extern "C" fn event_trampoline(code: c_int, message: *const c_char, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let state = unsafe { &*(user_data as *const CallbackState) };
    let msg = if !message.is_null() {
        unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .to_string()
    } else {
        String::new()
    };
    if let Ok(mut guard) = state.on_event.lock() {
        (guard.as_mut())(code, &msg);
    }
}

impl SerialPort {
    pub fn open(path: &str) -> Result<Self, i32> {
        let c_path = CString::new(path).map_err(|_| libc::EINVAL)?;
        let mut handle: *mut ms_serial_port = ptr::null_mut();
        let rc = unsafe { ms_serial_port_open(c_path.as_ptr(), &mut handle) };
        if rc != 0 {
            return Err(rc);
        }
        Ok(Self {
            handle,
            callbacks: None,
        })
    }

    pub fn configure(&mut self, config: &SerialConfig) -> Result<(), i32> {
        let raw = config.to_raw();
        let rc = unsafe { ms_serial_port_configure(self.handle, &raw) };
        if rc != 0 {
            return Err(rc);
        }
        Ok(())
    }

    pub fn start<F, E>(&mut self, data_cb: F, event_cb: E) -> Result<(), i32>
    where
        F: FnMut(&[u8]) + Send + 'static,
        E: FnMut(i32, &str) + Send + 'static,
    {
        let state = Arc::new(CallbackState {
            on_data: Mutex::new(Box::new(data_cb)),
            on_event: Mutex::new(Box::new(event_cb)),
        });
        let callbacks = ms_serial_callbacks {
            on_data: Some(data_trampoline),
            on_event: Some(event_trampoline),
        };
        let rc = unsafe {
            ms_serial_port_start(self.handle, callbacks, Arc::as_ptr(&state) as *mut c_void)
        };
        if rc != 0 {
            return Err(rc);
        }
        self.callbacks = Some(state);
        Ok(())
    }

    pub fn stop(&mut self) {
        unsafe { ms_serial_port_stop(self.handle) };
        self.callbacks = None;
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, i32> {
        let rc = unsafe { ms_serial_port_write(self.handle, data.as_ptr(), data.len()) };
        if rc < 0 {
            return Err(rc as i32);
        }
        Ok(rc as usize)
    }
}

impl Drop for SerialPort {
    fn drop(&mut self) {
        unsafe {
            ms_serial_port_stop(self.handle);
            ms_serial_port_close(self.handle);
        }
    }
}

fn default_config_raw() -> ms_serial_config {
    ms_serial_config {
        baud_rate: 115_200,
        data_bits: 8,
        stop_bits: 1,
        parity: ms_serial_parity_MS_SERIAL_PARITY_NONE,
        flow_control: ms_serial_flow_control_MS_SERIAL_FLOW_NONE,
        rx_buffer_size: 1 << 15,
        tx_buffer_size: 1 << 15,
        read_timeout_ms: 100,
        write_timeout_ms: 100,
    }
}

#[allow(dead_code)]
pub fn default_config() -> SerialConfig {
    SerialConfig::default()
}

pub fn list_serial_ports() -> Result<Vec<SerialDevice>, i32> {
    let mut raw_list: *mut ms_serial_port_info = ptr::null_mut();
    let mut count: usize = 0;
    let rc = unsafe { ms_serial_port_enumerate(&mut raw_list, &mut count) };
    if rc != 0 {
        return Err(rc);
    }
    let slice = unsafe { slice::from_raw_parts(raw_list, count) };
    let mut devices = Vec::with_capacity(slice.len());
    for item in slice {
        let path = unsafe { CStr::from_ptr(item.path.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let description = unsafe { CStr::from_ptr(item.description.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        devices.push(SerialDevice { path, description });
    }
    unsafe { ms_serial_port_list_free(raw_list, count) };
    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serial_config_roundtrip() {
        let mut cfg = SerialConfig::default();
        cfg.baud_rate = 230_400;
        cfg.data_bits = 7;
        cfg.stop_bits = StopBits::Two;
        cfg.parity = Parity::Even;
        cfg.flow_control = FlowControl::RtsCts;

        let raw = cfg.to_raw();
        let restored = SerialConfig::from_raw(raw);
        assert_eq!(restored.baud_rate, 230_400);
        assert_eq!(restored.data_bits, 7);
        assert_eq!(restored.stop_bits, StopBits::Two);
        assert_eq!(restored.parity, Parity::Even);
        assert_eq!(restored.flow_control, FlowControl::RtsCts);
    }
}
