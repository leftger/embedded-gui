# Screen transition presets

`embedded-gui` provides named transition presets and timing helpers for common shell navigation patterns on fixed-frame embedded displays.

## Quick use

```rust
use embedded_gui::prelude::*;

let spec = TransitionPreset::WindowPush.spec();
// or: ScreenTransitionSpec::push_moook(MOOOK_DURATION_MS)

runner.apply(&mut stack, ScreenCommand::Push(id), spec, &mut events)?;
```

## Catalog

| `TransitionPreset` | `ScreenTransitionEffect` |
|---|---|
| `WindowPush` | `PushMoook` |
| `WindowPop` | `PopMoook` |
| `WindowPushRound` | `PortHoleLeft` |
| `WindowPopRound` | `PortHoleRight` |
| `Shutter*` | `Shutter*` |
| `RoundFlip*` | `RoundFlip*` |
| `PortHole*` | `PortHole*` |
| `ModalPresent` / `ModalDismiss` | `ModalSlideUp` / `ModalSlideDown` |
| `TimelineSlide` | `SlideLeft` |
| `Fade` | `Fade` |

## Timing

- `MOOOK_DURATION_MS` — window push/pop spatial curve (7 frames @ 30 Hz)
- `SHUTTER_DURATION_MS` / `PORT_HOLE_DURATION_MS` — 6 frames (198 ms)
- `Easing::Moook` — `moook_curve` for property animations

Vector asset–driven modal/dot/launcher sequences are not supported; rectangular clip and slide presets cover typical stack navigation.
