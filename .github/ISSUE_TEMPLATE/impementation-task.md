---
name: Implementation Task
about: Implementation task for CTD crash reporter
title: "feat: [Brief Description]"
labels: enhancement
assignees: ''
---

## Goal

**Single, focused objective this task achieves.**

## Implementation Requirements

### Required Changes
- [ ] Change 1
- [ ] Change 2

### Affected Crates/Mods
- `lib/ctd-core` - Core crash reporting library
- `mods/<mod-name>` - Game-specific plugin

### API/Interface Changes
```rust
// New or modified public interfaces
```

### Error Handling
- What errors to return and when
- Use `CtdError` variants from ctd-core

## Acceptance Criteria

### Tests Required
```rust
#[test]
fn test_feature() {
    // Test case description
}
```

### Build Requirements
- [ ] `cargo build --workspace` passes
- [ ] `cargo clippy --workspace` passes with no warnings
- [ ] `cargo test --workspace` passes

## What NOT to Include

- Out of scope item 1
- Future consideration item 2

## File Locations

- Implementation: `lib/ctd-core/src/...`
- Tests: Inline or `lib/ctd-core/src/.../tests.rs`
- Game plugin: `mods/<game>/src/...`

## Success Criteria

- [ ] All tests pass
- [ ] Clippy passes with no warnings
- [ ] CI build passes
- [ ] Documentation updated if public API changed
- [ ] CHANGELOG.md updated under `[Unreleased]` section

**This issue is complete when:** [Specific, measurable completion condition]

## Context & References

- Related issues: #
- Docs:
