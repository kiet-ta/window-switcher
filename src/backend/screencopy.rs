use wayland_client::{Connection, Dispatch, QueueHandle, delegate_noop, WEnum};
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_shm::{self, WlShm};
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1};

use std::os::unix::io::RawFd;
use libc::{memfd_create, MFD_CLOEXEC};
use std::ffi::CString;
use memmap2::MmapMut;

/// Abstracts the safe POSIX file-descriptor lifecycle logic for interacting with
/// `wl_shm` Shared Memory pools exposed by Wayland Protocol.
#[allow(dead_code)]
pub struct SHMBuffer {
    pub fd: RawFd,
    pub mmap: MmapMut,
    pub size: usize,
}

impl SHMBuffer {
    pub fn new(size: usize) -> Result<Self, String> {
        let name = CString::new("sw-screencopy-shm").unwrap();
        // Native Buffer Allocation via memfd abstraction
        let fd = unsafe { memfd_create(name.as_ptr(), MFD_CLOEXEC) };
        if fd < 0 {
            return Err("Failed to execute memfd_create natively.".into());
        }

        unsafe {
            if libc::ftruncate(fd, size as libc::off_t) < 0 {
                libc::close(fd);
                return Err("Failed to truncate allocated SHM buffer block.".into());
            }
        }

        let mmap = unsafe {
            MmapMut::map_mut(fd).map_err(|e| {
                libc::close(fd);
                format!("Failed to securely memory-map buffer: {}", e)
            })?
        };

        Ok(Self { fd, mmap, size })
    }
}

// Absolute Memory Safety: Guaranteeing strict release of underlying OS Handles
impl Drop for SHMBuffer {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

pub enum FrameState {
    Init,
    Buffer(wl_shm::Format, u32, u32, u32),
    Ready,
    Failed,
}

#[allow(dead_code)]
pub struct CaptureState {
    pub registry: Option<WlRegistry>,
    pub shm: Option<WlShm>,
    pub outputs: Vec<WlOutput>,
    pub output_names: Vec<(WlOutput, String)>,
    pub screencopy_manager: Option<ZwlrScreencopyManagerV1>,
    pub frame_state: FrameState,
}

impl Dispatch<WlRegistry, ()> for CaptureState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wayland_client::protocol::wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wayland_client::protocol::wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                "wl_shm" => {
                    let shm = registry.bind::<WlShm, _, _>(name, 1, qh, ());
                    state.shm = Some(shm);
                }
                "wl_output" => {
                    let v = std::cmp::min(version, 4);
                    let output = registry.bind::<WlOutput, _, _>(name, v, qh, ());
                    state.outputs.push(output);
                }
                "zwlr_screencopy_manager_v1" => {
                    let manager = registry.bind::<ZwlrScreencopyManagerV1, _, _>(name, 1, qh, ());
                    state.screencopy_manager = Some(manager);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<ZwlrScreencopyFrameV1, ()> for CaptureState {
    fn event(
        state: &mut Self,
        _: &ZwlrScreencopyFrameV1,
        event: zwlr_screencopy_frame_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_screencopy_frame_v1::Event::Buffer { format, width, height, stride } => {
                if let WEnum::Value(fmt) = format {
                    state.frame_state = FrameState::Buffer(fmt, width, height, stride);
                }
            }
            zwlr_screencopy_frame_v1::Event::Ready { tv_sec_hi: _, tv_sec_lo: _, tv_nsec: _ } => {
                state.frame_state = FrameState::Ready;
            }
            zwlr_screencopy_frame_v1::Event::Failed => {
                state.frame_state = FrameState::Failed;
            }
            _ => {}
        }
    }
}

impl Dispatch<WlOutput, ()> for CaptureState {
    fn event(
        state: &mut Self,
        output: &WlOutput,
        event: wayland_client::protocol::wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wayland_client::protocol::wl_output::Event::Name { name } = event {
            state.output_names.push((output.clone(), name));
        }
    }
}

delegate_noop!(CaptureState: ignore WlShm);
delegate_noop!(CaptureState: ignore WlShmPool);
delegate_noop!(CaptureState: ignore WlBuffer);
delegate_noop!(CaptureState: ignore ZwlrScreencopyManagerV1);

/// Dispatches a true screencopy frame request for the active output.
/// Orchestrates the entire `wayland-client` v0.31 protocol cycle natively.
pub async fn capture_active_workspace(out_path: &str, target_monitor_name: &str) -> Result<(), String> {
    let conn = Connection::connect_to_env().map_err(|e| format!("Wayland connect error: {}", e))?;
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());

    let mut state = CaptureState {
        registry: None,
        shm: None,
        outputs: Vec::new(),
        output_names: Vec::new(),
        screencopy_manager: None,
        frame_state: FrameState::Init,
    };

    // Roundtrip to establish registry globals binding
    event_queue.roundtrip(&mut state).map_err(|e| format!("Roundtrip error: {}", e))?;

    let manager = state.screencopy_manager.clone().ok_or("wlr-screencopy not supported by compositor")?;
    let wl_shm = state.shm.clone().ok_or("wl_shm not supported")?;

    // Second Roundtrip: Await bound wl_output objects to broadcast their details natively.
    event_queue.roundtrip(&mut state).map_err(|e| format!("Roundtrip error: {}", e))?;

    let mut target_output = None;
    for (out, name) in &state.output_names {
        if name == target_monitor_name {
            target_output = Some(out.clone());
            break;
        }
    }
    
    // Multi-Monitor Strategy: Failover to primary indexed output if the target identity dynamically detached.
    let output = target_output.or_else(|| state.outputs.first().cloned()).ok_or("No output found mapping to target.")?;

    let frame = manager.capture_output(0, &output, &qh, ());

    // Await the wlr composite framework to declare frame Buffer sizes
    event_queue.roundtrip(&mut state).map_err(|e| format!("Roundtrip error: {}", e))?;

    let (format, width, height, stride) = match state.frame_state {
        FrameState::Buffer(f, w, h, s) => (f, w, h, s),
        _ => return Err("Failed to get buffer event from compositor".into()),
    };

    let size = (height * stride) as usize;
    let shm_buffer = SHMBuffer::new(size)?;
    
    let fd = unsafe { std::os::unix::io::BorrowedFd::borrow_raw(shm_buffer.fd) };
    let pool = wl_shm.create_pool(fd, size as i32, &qh, ());
    let wl_buffer = pool.create_buffer(0, width as i32, height as i32, stride as i32, format, &qh, ());

    frame.copy(&wl_buffer);

    // Synchronously listen until the graphics block writes to our Mmap explicitly
    loop {
        event_queue.blocking_dispatch(&mut state).map_err(|e| format!("Dispatch error: {}", e))?;
        match state.frame_state {
            FrameState::Ready => break,
            FrameState::Failed => return Err("Compositor failed to copy frame".into()),
            _ => continue,
        }
    }

    crate::backend::image_processor::process_and_save(
        &shm_buffer.mmap,
        width,
        height,
        stride,
        out_path
    );

    Ok(())
}
