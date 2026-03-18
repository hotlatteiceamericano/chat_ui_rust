# TODOS

## Architecture

### Fix hardcoded User-Id header in WebSocket connection

**What:** Pass `current_user_id` to `connect_websocket()` so the WS server receives the actual user ID.

**Why:** Currently `connect_websocket()` hardcodes `User-Id: "0"` (main.rs:128), even though the user enters their real ID at startup. The server sees every client as user 0, which breaks message routing.

**Context:** `connect_websocket()` takes no parameters. The fix is to add a `user_id: u32` parameter, then use it in the `User-Id` header. The `current_user_id` value is available in `main()` on line 33 — just pass it through. Also review whether the empty `Authorization` header (line 123) should be populated.

**Effort:** S
**Priority:** P0
**Depends on:** None

### Replace .expect() calls in spawned tasks with graceful error handling

**What:** Replace all `.expect()` calls in spawned async tasks with error propagation through the `app_tx` channel so the app can shut down cleanly.

**Why:** Any WebSocket error (disconnect, malformed JSON, send failure) causes a panic inside a spawned task. Since `ratatui::restore()` only runs on the happy path in `main()`, the terminal is left in raw mode — the user sees garbled output and must run `reset`. This is the #1 reliability issue in the codebase.

**Context:** There are 6 `.expect()` calls across 3 spawned tasks in main.rs (lines 63, 66, 86, 89-90, 101-103). The fix involves: (1) adding an `Error` variant to `AppEvents` in app_event.rs, (2) replacing `.expect()` with `.send(AppEvents::Error { .. })` or logging + breaking the loop, (3) handling `AppEvents::Error` in the main event loop by setting `self.exit = true`. An alternative quick fix is installing a panic hook via `std::panic::set_hook` that calls `ratatui::restore()`, but this only fixes the terminal corruption — it doesn't prevent silent task death when a task panics without bringing down the process.

**Effort:** M
**Priority:** P1
**Depends on:** None

## Tests

### Add unit tests for state machine logic

**What:** Add `#[cfg(test)]` module with tests covering `handle_key_event`, `send_msg`, `handle_incoming_message`, and `move_up`/`move_down`.

**Why:** Zero tests exist in the project. The key event handler has 8+ branches (Ctrl+Q, Tab, Esc, j/k/Up/Down, Enter in list, Enter in chat, typing, backspace), `send_msg` has error paths (no conversation selected), and message routing groups by sender_id. All of this is testable without a terminal or WebSocket connection.

**Context:** To make `App` testable, you'll need to construct it with mock channels — `mpsc::unbounded_channel()` for both `app_rx` and `outbound_tx`. Key test cases to cover:
- Ctrl+Q sets `exit = true`
- Tab toggles focus only when `selected_conversation` is `Some`
- Tab does nothing when `selected_conversation` is `None`
- Esc returns focus to ConversationList
- j/Down at bottom of list stays at bottom
- k/Up at top of list stays at top
- Enter in list sets `selected_conversation` and focuses ChatInput
- Enter in ChatInput sends message to `outbound_tx` and stores locally
- `send_msg` with no selected conversation returns Err
- `handle_incoming_message` groups by `sender_id`
- Empty input buffer behavior on Enter

**Effort:** M
**Priority:** P1
**Depends on:** None
