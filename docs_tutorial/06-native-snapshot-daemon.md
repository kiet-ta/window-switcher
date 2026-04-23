# Native Snapshot Worker

Thumbnail capture is now handled by a dedicated worker thread.

## Trigger Model

1. The backend notices that the active window changed.
2. The previous active window address and monitor name are converted into a `ThumbnailJob`.
3. The worker reuses its existing Wayland screencopy session when possible.
4. The result is sent back to the backend as success or failure.
5. The backend updates `ThumbnailState` and emits a new `UiSnapshot` only if the UI output changed.

## Current Scope

- Preview files are still keyed by window address.
- The captured image still represents the active monitor/workspace view rather than a true per-window crop.
- Image writes are skipped when the PNG bytes have not changed.
