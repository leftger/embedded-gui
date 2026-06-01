# Interaction Behavior Contract

This document captures the expected interaction semantics for high-traffic widgets and shared input paths.

## Pointer Press/Release

- Pointer press targets the topmost clickable, visible, enabled widget under the cursor.
- Pointer release emits release events for the originally pressed widget.
- Release does not retarget to a different widget if the pointer ends outside the original hit rectangle.
- Scroll inertia can continue after release if drag velocity exceeds the configured threshold.
- Two pointer click cycles on the same widget within the configured pointer double-click window emit `DoubleClicked`.

## Dropdown

- `Select` toggles open/closed state on the focused dropdown.
- While open, `Up` and `Down` cycle selection.
- `Back` closes the focused open dropdown before emitting a global back event.
- Opening and closing emit `Opened` and `Closed` events respectively.

## Select Activation

- `Select` emits the standard activation path (`Pressed`, `Clicked`, `Activate`) for the focused widget.
- Two `Select` activations on the same focused widget within the configured double-select window
  emit `DoubleClicked` after the second activation.
- If the double-select window expires, the next `Select` starts a new click sequence.

## Raw Key Input Policy

- Widgets can opt into raw key semantics via per-widget key input policy.
- With `raw_select` enabled:
  - `SelectPressed` emits `Pressed` without immediate activation.
  - `SelectReleased` emits `Released`, then runs the standard select activation path.
- With `raw_back` enabled:
  - `BackPressed` emits `Pressed`.
  - `BackReleased` emits `Released`, then runs normal back behavior (including dropdown close-on-back).

## Per-widget Key Bindings

- Widgets can override `Select` and `Back` key behavior independently.
- Each key supports:
  - `Default`: preserve normal behavior
  - `Ignore`: consume key without action
  - `Activate`: run focused activation path
  - `Back`: run back action path

## Textarea

- Backspace/delete and insertion mutations emit `TextInput` and `ValueChanged`.
- No-op edit attempts do not emit mutation events.
  - Example: backspace at cursor 0 with no selection.
  - Example: delete-forward at end of text with no selection.
- Selection replacement semantics apply before insertion and deletion.
- Read-only textareas ignore mutation requests.

## Visual State Priority

Render-time visual state selection uses this priority:

1. `Pressed` when a pointer press is actively held on a widget.
2. `Focused` when the widget currently owns focus.
3. `Disabled` when the widget or an ancestor is disabled.
4. `Normal` otherwise.
