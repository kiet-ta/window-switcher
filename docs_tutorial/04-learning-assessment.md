# Learning Assessment

Use these questions to validate your understanding of the current architecture.

## 1. Why is `hyprctl -j` + `serde_json` safer than wrapper macros?

**Expected reasoning:**  
Because command failures and parse failures are represented as `Result`/`Option`, we can skip a refresh cycle without panicking the process.

## 2. Why keep `EventListener` but remove `hyprland::data` usage?

**Expected reasoning:**  
`EventListener` is useful as a lightweight signal trigger. Data fetching is the risky part, so it was moved to explicit OS commands for full control of error handling.

## 3. Why does focus dispatch run on a worker queue?

**Expected reasoning:**  
To prevent GTK input stalls. Enter should close the overlay immediately even if compositor IPC is delayed.

## 4. What does "silent failover" mean in `refresh_and_send`?

**Expected reasoning:**  
If any command or JSON parse fails, the function ignores that segment and continues running future cycles. The app stays alive.

## 5. Why still keep logs for focus dispatch?

**Expected reasoning:**  
Silent failover is for non-critical read paths. Focus action is user-triggered and operationally important, so we keep observability in `/tmp/window-switcher-focus.log`.

## 6. What failure can still happen without crashing?

**Expected reasoning:**  
Malformed compositor JSON, missing `hyprctl`, monitor lookup mismatch, or temporary I/O errors. All should degrade behavior, not terminate the process.
