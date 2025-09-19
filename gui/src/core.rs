use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;
use std::sync::{Arc, Mutex};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct SerialPort {
    handle: *mut ms_serial_port,
    callbacks: Option<Arc<CallbackState>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SerialDevice {
    pub path: String,
    pub description: String,
}

struct CallbackState {
    on_data: Mutex<Box<dyn FnMut(&[u8]) + Send + 'static>>,
    on_event: Mutex<Box<dyn FnMut(i32, &str) + Send + 'static>>,
}

unsafe extern "C" fn data_trampoline(data: *const u8, length: usize, user_data: *mut c_void) {
    if data.is_null() || user_data.is_null() {
        return;
    }
    let slice = slice::from_raw_parts(data, length);
    let state = &*(user_data as *const CallbackState);
    if let Ok(mut guard) = state.on_data.lock() {
        (guard.as_mut())(slice);
    }
}

unsafe extern "C" fn event_trampoline(code: c_int, message: *const c_char, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let state = &*(user_data as *const CallbackState);
    let msg = if !message.is_null() {
        CStr::from_ptr(message).to_string_lossy().to_string()
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

    pub fn configure(&mut self, config: &ms_serial_config) -> Result<(), i32> {
        let rc = unsafe { ms_serial_port_configure(self.handle, config) };
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

pub fn default_config() -> ms_serial_config {
    ms_serial_config {
        baud_rate: 115200,
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
