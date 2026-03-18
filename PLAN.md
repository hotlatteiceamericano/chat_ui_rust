# Implementation Plan — Real-time TUI Chat

**Review mode:** SELECTIVE EXPANSION
**Date:** 2026-03-18
**Branch:** main

---

## Implemented (this session)

| # | Change | File |
|---|---|---|
| 1 | `move_down()` usize underflow crash fix — guard `if conversations.is_empty()` | `app.rs:137` |
| 2 | Empty message guard in `send_msg()` — no blank messages sent to server | `app.rs:262` |
| 3 | Cursor indicator in input box — `frame.set_cursor_position()` when ChatInput focused | `app.rs:253` |

---

## Open Risks (deferred)

| Priority | Issue | Location |
|---|---|---|
| P0 | `User-Id` header hardcoded as `"0"` — server sees every client as user 0, routing broken | `main.rs:128` |
| P1 | 6 `.expect()` calls in spawned tasks → panic leaves terminal in raw mode | `main.rs:63,66,86,89-90,101-103` |
| P1 | Zero tests — see TODOS.md for the 11 test cases to add | — |

---

## Cherry-picks Considered but Skipped

- New conversation modal (Ctrl+N) — designed in `design_doc.md`, not implemented; hardcoded list stays for now
- Message color differentiation — ░/▓ + alignment is enough for now
- Auto-add unknown senders to conversation list
- Scroll history (PageUp/PageDown in chat panel)
- WebSocket URL configurability (env var / CLI arg)
- App struct refactor — extract `render_sidebar()` / `render_chat_panel()` from monolithic `draw()`
- Test suite — deferred to a future session

---

## Architecture Notes

**Messages data model** — `HashMap<u32, Vec<Message>>` keyed by user_id — is a **one-way door**
for 1:1 chat. Group chats would require changing to `HashMap<ConversationId, Vec<Message>>`.
Name this decision before adding any room/group feature.

**`conversations: Vec<User>` + `selected_conversation: Option<usize>`** — the index becomes
stale if the list is ever re-sorted or filtered. Acceptable while the list is static/hardcoded;
revisit before adding dynamic user discovery.

**External dep risk** — `chat_websocket_service_rust` is pinned to git `main` with no version
tag. A push to that repo can silently break this build.

---

## 12-Month Ideal (for reference)

- Dynamic user/room list from server
- Persistent message history (SQLite or file)
- Unread message indicators
- WebSocket reconnection on disconnect
- Full test suite
- Connection status display
- Search
