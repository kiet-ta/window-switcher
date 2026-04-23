use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_shm::{self, WlShm};
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, WEnum, delegate_noop};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::{
    self, ZwlrScreencopyFrameV1,
};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;

use libc::{MFD_CLOEXEC, memfd_create};
use memmap2::MmapMut;
use std::ffi::CString;
use std::os::unix::io::RawFd;

pub struct SHMBuffer {
    pub fd: RawFd,
    pub mmap: MmapMut,
}

impl SHMBuffer {
    pub fn new(size: usize) -> Result<Self, String> {
        let name = CString::new("sw-screencopy-shm").map_err(|error| error.to_string())?;
        let fd = unsafe { memfd_create(name.as_ptr(), MFD_CLOEXEC) };
        if fd < 0 {
            return Err("Failed to execute memfd_create.".into());
        }

        unsafe {
            if libc::ftruncate(fd, size as libc::off_t) < 0 {
                libc::close(fd);
                return Err("Failed to resize SHM buffer.".into());
            }
        }

        let mmap = unsafe {
            MmapMut::map_mut(fd).map_err(|error| {
                libc::close(fd);
                format!("Failed to memory-map buffer: {error}")
            })?
        };

        Ok(Self { fd, mmap })
    }
}

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

pub struct CaptureState {
    pub shm: Option<WlShm>,
    pub outputs: Vec<WlOutput>,
    pub output_names: Vec<(WlOutput, String)>,
    pub screencopy_manager: Option<ZwlrScreencopyManagerV1>,
    pub frame_state: FrameState,
}

pub struct ScreencopySession {
    connection: Connection,
    event_queue: EventQueue<CaptureState>,
    queue_handle: QueueHandle<CaptureState>,
    state: CaptureState,
    shm: WlShm,
    manager: ZwlrScreencopyManagerV1,
}

impl ScreencopySession {
    pub fn connect() -> Result<Self, String> {
        let connection =
            Connection::connect_to_env().map_err(|error| format!("Wayland connect error: {error}"))?;
        let display = connection.display();
        let mut event_queue = connection.new_event_queue();
        let queue_handle = event_queue.handle();
        let _registry = display.get_registry(&queue_handle, ());

        let mut state = CaptureState {
            shm: None,
            outputs: Vec::new(),
            output_names: Vec::new(),
            screencopy_manager: None,
            frame_state: FrameState::Init,
        };

        event_queue
            .roundtrip(&mut state)
            .map_err(|error| format!("Roundtrip error: {error}"))?;
        event_queue
            .roundtrip(&mut state)
            .map_err(|error| format!("Roundtrip error: {error}"))?;

        let shm = state.shm.clone().ok_or("wl_shm not supported")?;
        let manager = state
            .screencopy_manager
            .clone()
            .ok_or("wlr-screencopy not supported by compositor")?;

        Ok(Self {
            connection,
            event_queue,
            queue_handle,
            state,
            shm,
            manager,
        })
    }

    pub fn capture_active_workspace(
        &mut self,
        out_path: &str,
        target_monitor_name: &str,
    ) -> Result<(), String> {
        self.state.frame_state = FrameState::Init;
        self.event_queue
            .roundtrip(&mut self.state)
            .map_err(|error| format!("Roundtrip error: {error}"))?;

        let output = self.find_output(target_monitor_name)?;
        let frame = self.manager.capture_output(0, &output, &self.queue_handle, ());

        self.event_queue
            .roundtrip(&mut self.state)
            .map_err(|error| format!("Roundtrip error: {error}"))?;

        let (format, width, height, stride) = match self.state.frame_state {
            FrameState::Buffer(format, width, height, stride) => (format, width, height, stride),
            _ => return Err("Failed to get screencopy buffer event.".into()),
        };

        let size = (height * stride) as usize;
        let shm_buffer = SHMBuffer::new(size)?;
        let fd = unsafe { std::os::unix::io::BorrowedFd::borrow_raw(shm_buffer.fd) };
        let pool = self.shm.create_pool(fd, size as i32, &self.queue_handle, ());
        let wl_buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            format,
            &self.queue_handle,
            (),
        );

        frame.copy(&wl_buffer);

        loop {
            self.event_queue
                .blocking_dispatch(&mut self.state)
                .map_err(|error| format!("Dispatch error: {error}"))?;

            match self.state.frame_state {
                FrameState::Ready => break,
                FrameState::Failed => return Err("Compositor failed to copy frame.".into()),
                _ => {}
            }
        }

        crate::backend::image_processor::process_and_save(
            &shm_buffer.mmap,
            width,
            height,
            stride,
            out_path,
        )
    }

    fn find_output(&mut self, target_monitor_name: &str) -> Result<WlOutput, String> {
        if let Some((output, _)) = self
            .state
            .output_names
            .iter()
            .rev()
            .find(|(_, name)| name == target_monitor_name)
        {
            return Ok(output.clone());
        }

        self.state
            .outputs
            .first()
            .cloned()
            .ok_or("No output found for screencopy.".into())
    }

    #[allow(dead_code)]
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
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
        if let wayland_client::protocol::wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_shm" => {
                    state.shm = Some(registry.bind::<WlShm, _, _>(name, 1, qh, ()));
                }
                "wl_output" => {
                    let version = std::cmp::min(version, 4);
                    state.outputs.push(registry.bind::<WlOutput, _, _>(name, version, qh, ()));
                }
                "zwlr_screencopy_manager_v1" => {
                    state.screencopy_manager = Some(
                        registry.bind::<ZwlrScreencopyManagerV1, _, _>(name, 1, qh, ()),
                    );
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
            zwlr_screencopy_frame_v1::Event::Buffer {
                format,
                width,
                height,
                stride,
            } => {
                if let WEnum::Value(format) = format {
                    state.frame_state = FrameState::Buffer(format, width, height, stride);
                }
            }
            zwlr_screencopy_frame_v1::Event::Ready {
                tv_sec_hi: _,
                tv_sec_lo: _,
                tv_nsec: _,
            } => state.frame_state = FrameState::Ready,
            zwlr_screencopy_frame_v1::Event::Failed => state.frame_state = FrameState::Failed,
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
