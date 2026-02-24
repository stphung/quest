# PAT Update Flow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Allow users to update their GitHub PAT in-place when it expires, without unlinking and relinking.

**Architecture:** Auth error detection at the `ureq::Error` level distinguishes 401/403 from other failures, surfacing a `TokenExpired` status. A new `[P]` key opens a minimal token-input dialog that validates, saves, and re-creates the git remote with the new credentials. The flow reuses the existing masked-input UI pattern and background-thread channel model.

**Tech Stack:** Rust, ureq 3.2 (HTTP), git2 (remote management), Ratatui (TUI rendering)

**Design doc:** `docs/plans/2026-02-24-pat-update-flow-design.md`

---

### Task 1: Add `CloudStatus::TokenExpired` and `is_auth_error()` helper

**Files:**
- Modify: `src/history/cloud.rs:44-56` (CloudStatus enum)
- Modify: `src/history/cloud.rs` (add helper function after line 783)

**Step 1: Add `TokenExpired` variant to `CloudStatus`**

In `src/history/cloud.rs`, add `TokenExpired` after `OutOfSync`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudStatus {
    Offline,
    Linked,
    Syncing,
    OutOfSync,
    /// The stored PAT is expired or revoked (HTTP 401/403).
    TokenExpired,
    Error(String),
}
```

**Step 2: Add `is_auth_error()` helper function**

Add after the tests module (or before, near the internal helpers section ~line 760):

```rust
/// Check whether an error message indicates an authentication failure (expired/revoked PAT).
///
/// Matches HTTP 401/403 status codes and common GitHub auth error strings from ureq.
pub fn is_auth_error(error_msg: &str) -> bool {
    // ureq formats status errors as "https://...: status code 401"
    error_msg.contains("status code 401")
        || error_msg.contains("status code 403")
        || error_msg.contains("Bad credentials")
        || error_msg.contains("401")
            && (error_msg.contains("Unauthorized") || error_msg.contains("auth"))
}
```

**Step 3: Add `update_token()` function to cloud.rs**

Add after `unlink()` (~line 656):

```rust
/// Update the PAT for an existing cloud link.
///
/// Validates the new token, updates the saved config, and re-creates the
/// git remote with the new auth URL. The existing repo link is preserved.
pub fn update_token(quest_dir: &Path, new_token: &str, config: &CloudConfig) -> Result<CloudConfig, String> {
    // 1. Validate the new token.
    let username = github_get_username(new_token)?;

    // 2. Re-create the git remote with updated credentials.
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;
    let auth_url = authenticated_url(&config.repo_url, new_token);

    if repo.find_remote(REMOTE_NAME).is_ok() {
        repo.remote_delete(REMOTE_NAME)
            .map_err(|e| format!("Failed to remove old remote: {e}"))?;
    }
    repo.remote(REMOTE_NAME, &auth_url)
        .map_err(|e| format!("Failed to add remote: {e}"))?;

    configure_fetch_refspec(&repo)?;

    // 3. Save updated config.
    let new_config = CloudConfig {
        token: new_token.to_string(),
        username,
        repo_url: config.repo_url.clone(),
    };
    save_config(quest_dir, &new_config)?;

    Ok(new_config)
}
```

**Step 4: Add `CloudOpResult::TokenUpdated` variant**

In the `CloudOpResult` enum (~line 59):

```rust
pub enum CloudOpResult {
    // ... existing variants ...
    /// Token was successfully updated. Contains the new config.
    TokenUpdated(CloudConfig),
    // ... Failed stays last ...
}
```

**Step 5: Run `cargo check`**

Run: `cargo check 2>&1 | head -40`
Expected: Compile errors about non-exhaustive match on `CloudStatus::TokenExpired` and `CloudOpResult::TokenUpdated` — this is expected, we'll fix those in later tasks.

**Step 6: Commit**

```bash
git add src/history/cloud.rs
git commit -m "feat(cloud): add TokenExpired status, is_auth_error(), update_token()"
```

---

### Task 2: Add `TokenExpired` status display and `[P]` footer hint in Time Vault UI

**Files:**
- Modify: `src/ui/time_vault_scene.rs:39-62` (BrowserMode enum)
- Modify: `src/ui/time_vault_scene.rs:993-1014` (paint_cloud_status)
- Modify: `src/ui/time_vault_scene.rs:1050-1090` (draw_controls footer)
- Modify: `src/ui/time_vault_scene.rs` (~line 774 area, add UpdatingToken dialog rendering)

**Step 1: Add `UpdatingToken` to `BrowserMode` enum**

```rust
pub enum BrowserMode {
    // ... existing variants ...
    ConfirmUnlink,
    /// Typing a new PAT to replace the expired/current one.
    UpdatingToken,
    DivergenceResolution,
}
```

**Step 2: Add `TokenExpired` rendering in `paint_cloud_status()`**

In the match on `state.cloud_status` (around line 993), add before the `Error` arm:

```rust
CloudStatus::TokenExpired => {
    ("\u{2601} \u{2717} token expired".to_string(), Color::Red)
}
```

**Step 3: Add `[P] Token` to the footer controls**

In `draw_controls()`, in the `Linked | OutOfSync | Error(_)` arm (~line 1058), add `TokenExpired` to the match and append the `[P] Token` hint:

```rust
CloudStatus::Linked
| CloudStatus::OutOfSync
| CloudStatus::TokenExpired
| CloudStatus::Error(_) => {
    // ... existing [C] Push · [V] Pull · [R] Repo hints ...
    spans.push(Span::styled(
        " \u{00b7} ",
        Style::default().fg(Color::Rgb(40, 80, 120)),
    ));
    spans.push(Span::styled("[P] ", Style::default().fg(Color::Cyan)));
    spans.push(Span::styled(
        "Token",
        Style::default().fg(Color::DarkGray),
    ));
}
```

**Step 4: Add `UpdatingToken` dialog rendering**

In the scene buffer rendering section (near the LinkingCloud dialog, ~line 774), add a new branch for `UpdatingToken`. Follow the same pattern as `LinkingCloud`:

```rust
BrowserMode::UpdatingToken => {
    put_text(buffer, cy, cx, "Update Token", Color::White);
    put_text(
        buffer,
        cy + 2,
        cx,
        "Paste a new GitHub Personal Access Token:",
        Color::DarkGray,
    );

    put_text(buffer, cy + 4, cx, "Token:", Color::DarkGray);
    let raw = &state.cloud_token_input;
    let masked = if raw.len() <= 4 {
        format!("{}_", raw)
    } else {
        let dots: String = "\u{2022}".repeat(raw.len() - 4);
        format!("{}{}_", dots, &raw[raw.len() - 4..])
    };
    put_text(buffer, cy + 4, cx + 7, &masked, Color::Yellow);

    if let Some(ref err) = state.cloud_token_error {
        put_text(buffer, cy + 6, cx, err, Color::Red);
    }

    put_text(buffer, cy + 8, cx, "[Enter] Update", Color::Cyan);
    put_text(buffer, cy + 8, cx + 17, "[Esc] Cancel", Color::DarkGray);
}
```

**Step 5: Add `UpdatingToken` to dialog exclusion lists**

Anywhere that `LinkingCloud` and `SelectingRepo` are listed as modes that suppress the normal controls/panels, add `UpdatingToken` alongside them. Search for patterns like:

```rust
BrowserMode::LinkingCloud | BrowserMode::SelectingRepo
```

And add `| BrowserMode::UpdatingToken` to each.

**Step 6: Run `cargo check`**

Run: `cargo check 2>&1 | head -40`
Expected: Remaining errors from non-exhaustive matches on `BrowserMode::UpdatingToken` in input handlers (fixed in Task 3).

**Step 7: Commit**

```bash
git add src/ui/time_vault_scene.rs
git commit -m "feat(ui): add TokenExpired display, [P] Token hint, UpdatingToken dialog"
```

---

### Task 3: Add `[P]` key handler and `handle_updating_token()` in input

**Files:**
- Modify: `src/input/time_vault_input.rs:8-46` (TimeVaultAction enum)
- Modify: `src/input/time_vault_input.rs:56-69` (dispatch match)
- Modify: `src/input/time_vault_input.rs:199-254` (handle_browse, add [P] key)
- Add: `handle_updating_token()` function in same file

**Step 1: Add `UpdateToken` variant to `TimeVaultAction`**

```rust
pub enum TimeVaultAction {
    // ... existing variants ...
    /// Update the PAT (new token validated and ready to save).
    UpdateToken { token: String },
}
```

**Step 2: Add `UpdatingToken` to the dispatch match**

In `handle_time_vault_input()`, add:

```rust
BrowserMode::UpdatingToken => handle_updating_token(key, state),
```

**Step 3: Add `[P]` key handler in `handle_browse()`**

Add a new arm in `handle_browse()` after the `[X]` handler (~line 253):

```rust
KeyCode::Char('p') | KeyCode::Char('P') => {
    if state.focus == PanelFocus::Left {
        use crate::history::cloud::CloudStatus;
        let is_linked = matches!(
            &state.cloud_status,
            CloudStatus::Linked
                | CloudStatus::OutOfSync
                | CloudStatus::TokenExpired
                | CloudStatus::Error(_)
        );
        if is_linked {
            state.cloud_token_input.clear();
            state.cloud_token_error = None;
            state.mode = BrowserMode::UpdatingToken;
        }
    }
    TimeVaultAction::Continue
}
```

**Step 4: Implement `handle_updating_token()`**

Add the function after `handle_confirm_unlink()`:

```rust
fn handle_updating_token(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Esc => {
            state.cloud_token_input.clear();
            state.cloud_token_error = None;
            state.mode = BrowserMode::Browse;
            TimeVaultAction::Continue
        }
        KeyCode::Backspace => {
            state.cloud_token_input.pop();
            state.cloud_token_error = None;
            TimeVaultAction::Continue
        }
        KeyCode::Enter => {
            if state.cloud_token_input.is_empty() {
                state.cloud_token_error = Some("token cannot be empty".to_string());
                return TimeVaultAction::Continue;
            }
            let token = state.cloud_token_input.clone();
            state.cloud_token_input.clear();
            state.cloud_token_error = None;
            state.mode = BrowserMode::Browse;
            TimeVaultAction::UpdateToken { token }
        }
        KeyCode::Char(c) => {
            if state.cloud_token_input.len() < 100 {
                state.cloud_token_input.push(c);
            }
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}
```

**Step 5: Run `cargo check`**

Run: `cargo check 2>&1 | head -40`
Expected: Errors about `TimeVaultAction::UpdateToken` not handled in `input/mod.rs` (Task 4).

**Step 6: Commit**

```bash
git add src/input/time_vault_input.rs
git commit -m "feat(input): add [P] key handler and handle_updating_token()"
```

---

### Task 4: Wire `UpdateToken` through InputResult and input_routing.rs

**Files:**
- Modify: `src/input/types.rs:103-153` (InputResult enum)
- Modify: `src/input/mod.rs:130-191` (TimeVaultAction dispatch)
- Modify: `src/main_helpers/input_routing.rs:104-114` (exhaustive match)

**Step 1: Add `UpdateToken` variant to `InputResult`**

In `src/input/types.rs`, add after `ResolveKeepBoth`:

```rust
/// Update the stored PAT with a new token.
UpdateToken { token: String },
```

**Step 2: Route `TimeVaultAction::UpdateToken` in `input/mod.rs`**

In the Time Vault action dispatch (around line 186), add:

```rust
TimeVaultAction::UpdateToken { token } => {
    return InputResult::UpdateToken { token };
}
```

**Step 3: Add `UpdateToken` to exhaustive match in `input_routing.rs`**

In `src/main_helpers/input_routing.rs`, add `InputResult::UpdateToken { .. }` to the cloud actions arm:

```rust
InputResult::ValidateToken { .. }
| InputResult::ChangeRepo
| InputResult::LinkCloud { .. }
| InputResult::PushCloud
| InputResult::PullCloud
| InputResult::UnlinkCloud
| InputResult::ResolveKeepLocal
| InputResult::ResolveUseCloud
| InputResult::ResolveKeepBoth
| InputResult::UpdateToken { .. } => InputAction::Continue,
```

**Step 4: Run `cargo check`**

Run: `cargo check 2>&1 | head -40`
Expected: Errors about `CloudOpResult::TokenUpdated` not handled in main.rs/character_screens.rs, and `InputResult::UpdateToken` not handled in main.rs (Task 5).

**Step 5: Commit**

```bash
git add src/input/types.rs src/input/mod.rs src/main_helpers/input_routing.rs
git commit -m "feat(input): wire UpdateToken through InputResult routing"
```

---

### Task 5: Handle `UpdateToken` action and `TokenUpdated` result in main.rs

**Files:**
- Modify: `src/main.rs` (~line 1269, after UnlinkCloud handler — add UpdateToken handler)
- Modify: `src/main.rs` (~line 577, CloudOpResult match — add TokenUpdated handler)

**Step 1: Handle `InputResult::UpdateToken` — spawn background thread**

In `src/main.rs`, find the section that handles cloud `InputResult` variants (after the `ResolveKeepBoth` handler, before the normal routing). Add:

```rust
InputResult::UpdateToken { token } => {
    if !cloud_op_in_flight {
        if let Some(ref config) = cloud_config {
            cloud_op_in_flight = true;
            cloud_status = history::cloud::CloudStatus::Syncing;
            if let GameOverlay::TimeVault { ref mut browser } = overlay {
                browser.cloud_status = cloud_status.clone();
            }
            let quest = quest_dir.clone();
            let cfg = config.clone();
            let tx = cloud_tx.clone();
            std::thread::spawn(move || {
                let result = match history::cloud::update_token(&quest, &token, &cfg) {
                    Ok(new_config) => history::cloud::CloudOpResult::TokenUpdated(new_config),
                    Err(e) => {
                        if history::cloud::is_auth_error(&e) {
                            history::cloud::CloudOpResult::Failed("invalid token".to_string())
                        } else {
                            history::cloud::CloudOpResult::Failed(e)
                        }
                    }
                };
                let _ = tx.send(result);
            });
        }
    }
    continue;
}
```

**Step 2: Handle `CloudOpResult::TokenUpdated` — update state**

In the `CloudOpResult` match in the main game loop (where `cloud_rx.try_recv()` is processed), add before `Failed`:

```rust
history::cloud::CloudOpResult::TokenUpdated(new_config) => {
    cloud_status = history::cloud::CloudStatus::Linked;
    cloud_username = Some(new_config.username.clone());
    cloud_config = Some(new_config);
}
```

**Step 3: Add auth error detection to existing cloud operations**

In the existing `Failed` handler in the `CloudOpResult` match, check for auth errors:

```rust
history::cloud::CloudOpResult::Failed(msg) => {
    if history::cloud::is_auth_error(&msg) {
        cloud_status = history::cloud::CloudStatus::TokenExpired;
    } else {
        cloud_status = history::cloud::CloudStatus::Error(msg.clone());
    }
    if let GameOverlay::TimeVault { ref mut browser } = overlay {
        browser.cloud_token_error = Some(msg);
    }
}
```

**Step 4: Update the `TokenExpired` match arms in [C] key handling**

In the section where `[C]` (Push) is handled, add `TokenExpired` alongside `Linked | OutOfSync | Error(_)` so push still works when status is token-expired (user might fix token first, then push):

Actually — when token is expired, push will fail. Better to NOT start a push. Instead, in the `handle_browse` input handler (already done in Task 3), the `[C]` key checks for cloud status. Update the existing `handle_browse` `[C]` match in `time_vault_input.rs` to include `TokenExpired`:

```rust
CloudStatus::Linked | CloudStatus::OutOfSync | CloudStatus::TokenExpired | CloudStatus::Error(_) => {
    state.mode = BrowserMode::ConfirmPush;
}
```

This is already covered by the existing match since `TokenExpired` would fall through. But verify that the [V] Pull and [R] Repo handlers also include `TokenExpired` in the `is_linked` check. They use `!matches!(&state.cloud_status, CloudStatus::Offline | CloudStatus::Syncing)` which naturally includes `TokenExpired`. Good — no change needed.

**Step 5: Run `cargo check`**

Run: `cargo check 2>&1 | head -40`
Expected: Errors from character_screens.rs (Task 6).

**Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat(cloud): handle UpdateToken action and TokenUpdated result in game loop"
```

---

### Task 6: Handle `UpdateToken` and `TokenUpdated` in character_screens.rs

**Files:**
- Modify: `src/main_helpers/character_screens.rs` (CloudOpResult match, TimeVaultAction handling)

**Step 1: Handle `CloudOpResult::TokenUpdated` in character select**

In the `CloudOpResult` match inside `update_time_vault_with_cloud_sync()` (or equivalent), add before `Failed`:

```rust
CloudOpResult::TokenUpdated(new_config) => {
    *cloud_username = Some(new_config.username.clone());
    *cloud_status = CloudStatus::Linked;
    *cloud_config = Some(new_config);
    if let Some(ref mut browser) = time_vault_browser {
        browser.cloud_status = cloud_status.clone();
        browser.cloud_username = cloud_username.clone();
    }
}
```

**Step 2: Handle auth errors in the existing `Failed` arm**

Update the `CloudOpResult::Failed` handler to check for auth errors:

```rust
CloudOpResult::Failed(msg) => {
    if history::cloud::is_auth_error(&msg) {
        *cloud_status = CloudStatus::TokenExpired;
    } else {
        *cloud_status = CloudStatus::Error(msg.clone());
    }
    // ... existing error display logic ...
}
```

**Step 3: Handle `TimeVaultAction::UpdateToken` in the action dispatcher**

Find where `TimeVaultAction` variants are matched and spawn threads (the function that handles browser input results). Add alongside the existing cloud action handlers:

```rust
TimeVaultAction::UpdateToken { token } => {
    if !*cloud_op_in_flight {
        if let Some(ref config) = cloud_config {
            *cloud_op_in_flight = true;
            *cloud_status = CloudStatus::Syncing;
            if let Some(ref mut browser) = time_vault_browser {
                browser.cloud_status = cloud_status.clone();
            }
            let quest = quest_dir.to_path_buf();
            let cfg = config.clone();
            let tx = cloud_tx.clone();
            std::thread::spawn(move || {
                let result = match history::cloud::update_token(&quest, &token, &cfg) {
                    Ok(new_config) => CloudOpResult::TokenUpdated(new_config),
                    Err(e) => {
                        if history::cloud::is_auth_error(&e) {
                            CloudOpResult::Failed("invalid token".to_string())
                        } else {
                            CloudOpResult::Failed(e)
                        }
                    }
                };
                let _ = tx.send(result);
            });
        }
    }
}
```

Note: The character_screens.rs file uses a slightly different pattern than main.rs — it may map `TimeVaultAction` directly rather than going through `InputResult`. Read the file carefully to match the existing pattern.

**Step 4: Run `cargo check`**

Run: `cargo check 2>&1 | head -40`
Expected: Clean compile (0 errors).

**Step 5: Run full CI checks**

Run: `make check`
Expected: All checks pass (fmt, clippy, tests, build, audit).

**Step 6: Commit**

```bash
git add src/main_helpers/character_screens.rs
git commit -m "feat(cloud): handle UpdateToken and TokenUpdated in character select"
```

---

### Task 7: Add auth error detection to all cloud operation threads

**Files:**
- Modify: `src/main.rs` (all cloud thread `Failed` error paths)
- Modify: `src/main_helpers/character_screens.rs` (all cloud thread `Failed` error paths)

This task ensures that ALL cloud operation failures (push, pull, fetch, validate, resolve) check for auth errors and set `TokenExpired` instead of generic `Error`.

**Step 1: Update push thread error path in main.rs**

Find the thread spawned for `PushCloud` and update its error handling:

```rust
Err(e) => {
    if history::cloud::is_auth_error(&e) {
        history::cloud::CloudOpResult::Failed("token expired".to_string())
    } else {
        history::cloud::CloudOpResult::Failed(e)
    }
}
```

**Step 2: Update pull thread error path in main.rs**

Same pattern for `PullCloud` thread.

**Step 3: Update resolve threads (KeepLocal, UseCloud, KeepBoth) in main.rs**

Same pattern for all three divergence resolution threads.

**Step 4: Repeat Steps 1-3 for character_screens.rs**

Apply the same auth-error-aware error handling to all cloud operation threads in character_screens.rs.

**Step 5: Ensure `CloudOpResult::Failed` handlers check `is_auth_error()`**

In both main.rs and character_screens.rs, verify that the `CloudOpResult::Failed` handler sets `TokenExpired` status when `is_auth_error()` returns true (done in Tasks 5-6).

**Step 6: Run full CI checks**

Run: `make check`
Expected: All checks pass.

**Step 7: Commit**

```bash
git add src/main.rs src/main_helpers/character_screens.rs
git commit -m "feat(cloud): detect auth errors in all cloud operations"
```

---

### Task 8: Add unit test for `is_auth_error()` and `update_token()` config handling

**Files:**
- Modify: `src/history/cloud.rs` (add tests to existing `#[cfg(test)]` module)

**Step 1: Write tests for `is_auth_error()`**

```rust
#[test]
fn is_auth_error_detects_401() {
    assert!(is_auth_error("https://api.github.com/user: status code 401"));
}

#[test]
fn is_auth_error_detects_403() {
    assert!(is_auth_error("GitHub API error: status code 403"));
}

#[test]
fn is_auth_error_detects_bad_credentials() {
    assert!(is_auth_error("Bad credentials"));
}

#[test]
fn is_auth_error_rejects_404() {
    assert!(!is_auth_error("status code 404"));
}

#[test]
fn is_auth_error_rejects_network_error() {
    assert!(!is_auth_error("connection refused"));
}
```

**Step 2: Run tests**

Run: `cargo test --lib -- cloud::tests -v`
Expected: All tests pass.

**Step 3: Commit**

```bash
git add src/history/cloud.rs
git commit -m "test: add unit tests for is_auth_error()"
```

---

### Task 9: Final verification and cleanup

**Step 1: Run full CI checks**

Run: `make check`
Expected: All checks pass (fmt, clippy, tests, build, audit).

**Step 2: Verify all `#[allow(dead_code)]` annotations are needed**

Check that `TimeVaultAction::ValidateToken` and `TimeVaultAction::LinkCloud` still have their `#[allow(dead_code)]` annotations (they're only used from the character select screen path, not the game screen path — the compiler may or may not flag them). Remove any that are no longer needed.

**Step 3: Verify no remaining TODO/FIXME from this feature**

Run: `grep -rn "TODO\|FIXME" src/history/cloud.rs src/ui/time_vault_scene.rs src/input/time_vault_input.rs`

**Step 4: Commit any cleanup**

```bash
git add -A && git commit -m "chore: cleanup dead_code annotations after PAT update feature"
```
