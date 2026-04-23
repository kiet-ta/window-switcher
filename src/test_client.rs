use wayland_client::{delegate_dispatch, delegate_noop, Connection, Dispatch, QueueHandle, WEnum};
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_buffer::{self, WlBuffer};
use wayland_client::protocol::wl_output::{self, WlOutput};
use wayland_client::protocol::wl_shm::{self, WlShm};
use wayland_client::protocol::wl_shm_pool::{self, WlShmPool};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_frame_v1::{
    self, ZwlrScreencopyFrameV1,
};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::{
    self, ZwlrScreencopyManagerV1,
};

#[derive(Debug)]
pub enum FrameState {
    Init,
    Buffer(wl_shm::Format, u32, u32, u32),
    Ready,
    Failed,
}

pub struct CaptureState {
    pub registry: Option<WlRegistry>,
    pub shm: Option<WlShm>,
    pub output: Option<WlOutput>,
    pub screencopy_manager: Option<ZwlrScreencopyManagerV1>,
    pub frame_state: FrameState,
}

impl Dispatch<WlRegistry, ()> for CaptureState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version: _,
        } = event
        {
            match interface.as_str() {
                "wl_shm" => {
                    let shm = registry.bind::<WlShm, _, _>(name, 1, qh, ());
                    state.shm = Some(shm);
                }
                "wl_output" => {
                    if state.output.is_none() {
                        let output = registry.bind::<WlOutput, _, _>(name, 1, qh, ());
                        state.output = Some(output);
                    }
                }
                "zwlr_screencopy_manager_v1" => {
                    let manager =
                        registry.bind::<ZwlrScreencopyManagerV1, _, _>(name, 1, qh, ());
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
            zwlr_screencopy_frame_v1::Event::Buffer {
                format,
                width,
                height,
                stride,
            } => {
                if let WEnum::Value(fmt) = format {
                    state.frame_state = FrameState::Buffer(fmt, width, height, stride);
                }
            }
            zwlr_screencopy_frame_v1::Event::Ready => {
                state.frame_state = FrameState::Ready;
            }
            zwlr_screencopy_frame_v1::Event::Failed => {
                state.frame_state = FrameState::Failed;
            }
            _ => {}
        }
    }
}

delegate_dispatch!(CaptureState: [WlRegistry: ()] => CaptureState);
delegate_dispatch!(CaptureState: [ZwlrScreencopyFrameV1: ()] => CaptureState);
delegate_noop!(CaptureState: ignore WlShm);
delegate_noop!(CaptureState: ignore WlShmPool);
delegate_noop!(CaptureState: ignore WlBuffer);
delegate_noop!(CaptureState: ignore WlOutput);
delegate_noop!(CaptureState: ignore ZwlrScreencopyManagerV1);

fn main() {}
