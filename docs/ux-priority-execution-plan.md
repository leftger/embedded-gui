# UX Priority Execution Plan

This plan tracks the selected priority sequence:

1. Motion tokens
2. State surfaces
3. Notification primitives
4. CardStory container
5. FeedTimeline
6. Menu contract

Status legend:
- [ ] not started
- [~] in progress
- [x] complete

---

## 1) Motion tokens
- Status: [~]
- Goal: reusable motion language that can be applied consistently across widgets, transitions, and app flows.
- Deliverables:
  - `MotionTokens` config object (durations, distances, opacity ranges, easing defaults).
  - Preset catalog (`focus_bump`, `settle`, `shake`, `stagger_in`, `toast_pop`, `peek_reveal`).
  - Helper APIs that consume tokens instead of hard-coded constants.
  - Example update showing token overrides.
- Acceptance criteria:
  - Presets compile without breaking existing API surface.
  - One place to tune global "feel" for animations.
  - Existing cinematic example can switch token set in <= 10 lines.
- Likely files:
  - `src/cinematic.rs`
  - `src/widget_animation.rs`
  - `src/transition_preset.rs`
  - `src/lib.rs`
  - `examples/cinematic_peek_glance_carddeck_showcase.rs`

## 2) State surfaces
- Status: [~]
- Goal: first-class surfaces for loading/empty/error/offline states with consistent visuals and interaction behavior.
- Deliverables:
  - `SurfaceState` enum (`Loading`, `Empty`, `Error`, `Offline`, `Ready`).
  - Reusable state surface widget or adapter layer.
  - Optional action affordances (`Retry`, `Open settings`, `Dismiss`) via callbacks/events.
  - Default icon/text layout with style overrides.
- Acceptance criteria:
  - App can switch state with one API call.
  - State surface supports no_std-friendly text/icon representation.
  - Keyboard/encoder navigation can focus retry action when available.
- Likely files:
  - `src/widgets/mod.rs`
  - `src/context.rs`
  - `src/input.rs`
  - `src/style.rs`
  - `src/lib.rs`
  - `examples/` new state surface demo

## 3) Notification primitives
- Status: [~]
- Goal: composable heads-up + action-sheet notification UX.
- Deliverables:
  - `HeadsUpBanner` primitive (ttl, auto-dismiss, manual dismiss).
  - `NotificationActionSheet` primitive (title/body/actions).
  - Event model for open/close/action-selected.
  - Preset choreography for enter/exit.
- Acceptance criteria:
  - Heads-up notification can animate in/out and auto-expire.
  - Action sheet supports at least 2 actions + cancel/back.
  - Works with pointer and key/encoder input paths.
- Likely files:
  - `src/widgets/mod.rs`
  - `src/context.rs`
  - `src/input.rs`
  - `src/cinematic.rs`
  - `src/lib.rs`
  - `examples/` new notifications demo

## 4) CardStory container
- Status: [~]
- Goal: summary -> detail -> graph narrative container with built-in transitions.
- Deliverables:
  - `CardStory` state container (current card index, direction, transitions).
  - Navigation helpers (next/prev/jump).
  - Integration with motion presets for card transitions.
  - Optional indicator (dots/progress/index label).
- Acceptance criteria:
  - Supports at least 3 cards with smooth forward/back transitions.
  - Back handling semantics are predictable and testable.
  - Demo app can build a card story in < 30 lines of setup.
- Likely files:
  - `src/cinematic.rs`
  - `src/context.rs`
  - `src/widgets/mod.rs`
  - `src/lib.rs`
  - `examples/cinematic_peek_glance_carddeck_showcase.rs` (or split example)

## 5) FeedTimeline
- Status: [~]
- Goal: feed/timeline primitive with compact rows, expanded preview, and detail pin behavior.
- Deliverables:
  - `FeedTimeline` widget model (items, selected index, scroll offset).
  - Row rendering variants (compact vs expanded).
  - Optional peek/pin transitions using existing cinematic helpers.
  - Event outputs for selected/opened/acted-on items.
- Acceptance criteria:
  - Scrolling and selection remain smooth under fixed capacity constraints.
  - Expanded item transitions do not break dirty-render assumptions.
  - Works with encoder-style incremental navigation.
- Likely files:
  - `src/widgets/mod.rs`
  - `src/context.rs`
  - `src/state.rs`
  - `src/input.rs`
  - `src/lib.rs`
  - `examples/` new feed timeline demo

## 6) Menu contract
- Status: [~]
- Goal: codify predictable menu semantics across components.
- Deliverables:
  - `MenuContract` policy (select/back/open/close/focus movement rules).
  - Shared key mapping defaults for list/menu/dropdown/action sheets.
  - Contract-focused tests for regression safety.
  - Documentation for expected UX behavior.
- Acceptance criteria:
  - Menu-like widgets use the same back/confirm semantics by default.
  - Contract can be overridden per-widget when needed.
  - Tests cover edge cases (top/bottom boundaries, nested open states, back behavior).
- Likely files:
  - `src/context.rs`
  - `src/input.rs`
  - `src/widget.rs`
  - `tests/gui.rs`
  - `docs/interaction-behavior-contract.md`

---

## Execution order
- Iteration A: (1) Motion tokens
- Iteration B: (2) State surfaces + (3) Notification primitives
- Iteration C: (4) CardStory + (5) FeedTimeline
- Iteration D: (6) Menu contract + regression hardening

## Notes
- Keep API additions additive where possible.
- Prefer preset-first APIs with sensible defaults, optional advanced overrides.
- Add at least one focused example per milestone to make UX intent visible quickly.
