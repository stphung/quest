> Backported design record. Sources: docs/plans/2026-02-24-pat-update-flow-design.md.

## 2026-02-24-pat-update-flow-design.md

# PAT Update Flow Design

**Date:** 2026-02-24
**Status:** Approved

## Problem

GitHub Personal Access Tokens expire. When a PAT expires, cloud operations fail with a generic error message. The only way to update the token is to unlink and re-link, which is clunky and non-obvious.

## Design

### 1. Auth Error Detection

When any cloud operation (push, pull, fetch, validate) fails, check whether the error indicates an authentication failure (HTTP 401/403). If so:

- Set `cloud_status` to a new `CloudStatus::TokenExpired` variant
- Display `"☁ ✗ token expired"` in the Time Vault status area
- Show `[P]` hint in the footer

For non-auth errors, behavior is unchanged (`CloudStatus::Error(msg)`).

### 2. [P] Update Token Flow

When linked, token-expired, or in an error state, pressing `[P]` opens a new `BrowserMode::UpdatingToken` dialog:

1. Reuses the existing masked token input UI (bullets + last 4 chars visible)
2. User pastes new PAT and presses Enter
3. Token is validated via `github_get_username()`
4. On success:
   - Update `token` in the in-memory `CloudConfig`
   - Save updated config to `~/.quest/.cloud.json`
   - Re-create the git remote with the new auth URL (PAT is embedded in the remote URL)
   - Set status to `CloudStatus::Linked`
5. On failure: show error message in the dialog, user can retry or Esc

No repo selection needed — the existing repo link is preserved.

### 3. Footer Hint Update

When linked/error/expired, the footer shows:
```
[C] Push · [V] Pull · [R] Repo · [P] Token
```

### What Doesn't Change

- `[X]` Unlink works as-is (full disconnect)
- `[C]` Link flow unchanged (first-time setup)
- No proactive startup token validation
- No encryption of stored token

## Implementation Scope

### New Types
- `CloudStatus::TokenExpired` variant
- `BrowserMode::UpdatingToken` variant
- `TimeVaultAction::UpdateToken { token }` variant
- `InputResult::UpdateToken { token }` variant
- `CloudOpResult::TokenUpdated(CloudConfig)` variant

### Modified Files
- `src/history/cloud.rs` — Add `TokenExpired` status, `update_token()` function, auth error detection helper
- `src/ui/time_vault_scene.rs` — Add `UpdatingToken` mode, `TokenExpired` status display, `[P]` footer hint
- `src/input/time_vault_input.rs` — Add `[P]` key handler, `handle_updating_token()` function
- `src/input/types.rs` — Add `UpdateToken` variant to `InputResult`
- `src/input/mod.rs` — Route `UpdateToken` action
- `src/main.rs` — Handle `UpdateToken` and `TokenUpdated` results
- `src/main_helpers/character_screens.rs` — Handle `UpdateToken` and `TokenUpdated` in character select
- `src/main_helpers/input_routing.rs` — Add `UpdateToken` to exhaustive match

### Auth Error Detection
Check `ureq::Error` variants for HTTP 401/403 status codes. Add a helper function `is_auth_error(error_msg: &str) -> bool` that pattern-matches on common auth failure strings from the GitHub API.
