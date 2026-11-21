// SPDX-License-Identifier: MIT OR Apache-2.0
//! ## Map
//! - Role: core/kit_hook_policy — Pure-Function Hook & Handoff Policy Brick (transpiled from `kit_hooks.py` & `handoff_controller.py`)
//! - Inputs: Shell command strings, file paths, prompt strings, and red-light state flags
//! - Outputs: Boolean predicates and deterministic policy codes (`u8`)
//! - Invariants:
//!   1. Zero external dependencies, zero mutable globals (`R09-2`), zero recursion (`R09-7`), nesting depth <= 2 (`R09-3`).
//!   2. `git commit --no-verify` / `-n` / `core.hooksPath` bypasses are unconditionally blocked.
//!   3. Auto-continue prompts (`[git-handoff 自動接續`) never reset human turn counters.

pub const PRE_BASH_ALLOW: u8 = 0;
pub const PRE_BASH_DENY_BYPASS: u8 = 1;
pub const PRE_BASH_WARN_DELETE: u8 = 2;

pub const PRE_EDIT_ALLOW: u8 = 0;
pub const PRE_EDIT_ASK_HOOK_CONFIG: u8 = 1;
pub const PRE_EDIT_DENY_RED_TEST: u8 = 2;

/// Returns `true` if a shell command attempts to bypass git hooks (`--no-verify`, `commit -n`, or `core.hooksPath`).
/// @outputs: 2
pub fn is_bypass_git_command(cmd: &str) -> bool {
    let lower = cmd.to_ascii_lowercase();
    if lower.contains("--no-verify") || lower.contains("core.hookspath") {
        return true;
    }
    lower.contains("git ") && lower.contains("commit") && lower.contains(" -n")
}

/// Returns `true` if `path` points to protected hook configuration files or directories.
/// @outputs: 2
pub fn is_protected_hook_path(path: &str) -> bool {
    let norm = path.replace('\\', "/").to_ascii_lowercase();
    norm.contains(".claude/settings.json")
        || norm.contains(".agents/hooks.json")
        || norm.contains(".githooks")
        || norm.contains("scripts/hooks")
}

/// Returns `true` if `path` matches a test file pattern (`tests/`, `_test.`, `.test.`, `test_*.py`, `.spec.`).
/// @outputs: 2
pub fn is_test_file_path(path: &str) -> bool {
    let norm = path.replace('\\', "/").to_ascii_lowercase();
    norm.contains("/tests/")
        || norm.starts_with("tests/")
        || norm.contains("/test/")
        || norm.contains("_test.")
        || norm.contains(".test.")
        || norm.contains(".spec.")
        || norm.contains("/test_")
}

/// Returns `true` if `text` is a machine-generated `[git-handoff 自動接續` prompt (must not reset human turn counters).
/// @outputs: 2
pub fn is_auto_continue_prompt(text: &str) -> bool {
    text.trim_start().starts_with("[git-handoff 自動接續")
}

/// Evaluates `PreToolUse` bash policy (`0` = allow, `1` = deny hook bypass, `2` = destructive/delete guard).
/// @outputs: 3
pub fn evaluate_pre_bash_policy(cmd: &str) -> u8 {
    if is_bypass_git_command(cmd) {
        return PRE_BASH_DENY_BYPASS;
    }
    let lower = cmd.to_ascii_lowercase();
    if lower.contains("rm ") || lower.contains("remove-item") || lower.contains("git clean") {
        return PRE_BASH_WARN_DELETE;
    }
    PRE_BASH_ALLOW
}

/// Evaluates `PreToolUse` edit policy (`0` = allow, `1` = ask before editing hook config, `2` = deny editing test during red light).
/// @outputs: 3
pub fn evaluate_pre_edit_policy(path: &str, red_light_active: bool) -> u8 {
    if is_protected_hook_path(path) {
        return PRE_EDIT_ASK_HOOK_CONFIG;
    }
    if red_light_active && is_test_file_path(path) {
        return PRE_EDIT_DENY_RED_TEST;
    }
    PRE_EDIT_ALLOW
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @outputs: 1
    #[test]
    fn test_kit_hook_policy_golden_vectors() {
        assert_eq!(evaluate_pre_bash_policy("cargo test"), PRE_BASH_ALLOW);
        assert_eq!(evaluate_pre_bash_policy("git commit -m 'msg' --no-verify"), PRE_BASH_DENY_BYPASS);
        assert_eq!(evaluate_pre_bash_policy("git commit -n -m x"), PRE_BASH_DENY_BYPASS);
        assert_eq!(evaluate_pre_bash_policy("git config core.hooksPath /tmp"), PRE_BASH_DENY_BYPASS);
        assert_eq!(evaluate_pre_bash_policy("rm -rf build/"), PRE_BASH_WARN_DELETE);

        assert_eq!(evaluate_pre_edit_policy("src/main.rs", false), PRE_EDIT_ALLOW);
        assert_eq!(evaluate_pre_edit_policy(".agents/hooks.json", false), PRE_EDIT_ASK_HOOK_CONFIG);
        assert_eq!(evaluate_pre_edit_policy("tests/test_a.py", false), PRE_EDIT_ALLOW);
        assert_eq!(evaluate_pre_edit_policy("tests/test_a.py", true), PRE_EDIT_DENY_RED_TEST);

        assert!(is_auto_continue_prompt("[git-handoff 自動接續] resume"));
        assert!(!is_auto_continue_prompt("請幫我測試"));
    }
}
