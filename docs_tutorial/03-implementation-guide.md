# Implementation Guide

## Backend Refresh Rules

- Active-window events are debounced for `75ms`.
- A `3s` safety poll remains to recover from missed compositor events.
- `clients -j` runs on each debounced refresh.
- `activewindow -j` runs once per refresh and accepts both `monitor` and `monitorID`.
- `monitors -j` is refreshed on startup, on cache miss, or every `10s`.

## MRU Ordering

- Each active window transition increments a local focus sequence.
- The current active window stays in the list.
- The list is sorted by descending `last_focus_seq`.
- The default selected item is the second result when two or more windows exist.

## Focus Worker

- UI input enqueues a `FocusCommand`.
- The worker executes `hyprctl dispatch focuswindow address:<addr>`.
- Duplicate pending requests for the same address are dropped.
- Failures are logged to `/tmp/window-switcher-focus.log`.

## Thumbnail Worker

- The backend queues a `ThumbnailJob` for the previously active address.
- Jobs are deduplicated per `(address, monitor_name)`.
- The worker reuses a `ScreencopySession`.
- Successful captures update per-window `ThumbnailState`.

## UI Rendering

- Cards are cached by window address.
- Reorder operations reuse the same card widgets.
- Label, preview state, active state, and selection state are updated in place.
- The status label makes loading, empty, and degraded backend modes explicit.
