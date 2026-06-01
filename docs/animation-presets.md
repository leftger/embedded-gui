# Animation Presets Cheat-Sheet

One-liner snippets for preset helpers in `embedded_gui::presets`.

```rust
use embedded_gui::prelude::*;
```

## Entrance

```rust
presets::entrance_fade_in_up(&mut animator, panel_id, 24, 12, 400)?;
```

## Attention

```rust
presets::attention_shake(&mut animator, button_id, 40, 3, 260)?;
```

## Style Motion

```rust
presets::style_breathe(&mut animator, card_id, 112, 220, 1, 4, 700)?;
```

```rust
presets::style_accent_cycle(
    &mut animator,
    card_id,
    Rgb565::new(0, 30, 20),
    Rgb565::new(31, 50, 0),
    800,
)?;
```

## Path Motion

```rust
presets::path_float_loop(&mut animator, icon_id, 80, 42, 3, 900)?;
```

## Orchestration

```rust
presets::orchestrate_stagger_x(&mut animator, &[a, b, c], 8, 28, 420, 60)?;
```

## Typical Frame Loop

```rust
animator.tick(16, &mut gui)?;
gui.render(&mut display)?;
```

## Custom Curves and Interpolators

`Animation` supports callback-driven motion shaping inspired by richer runtime animation systems.

```rust
fn overshoot_curve(t: f32) -> f32 {
    if t < 0.85 { t / 0.85 } else { 1.0 + (t - 0.85) * 0.4 }
}

let anim = Animation::new(0.0, 100.0, 500, Easing::Linear)
    .with_custom_curve(overshoot_curve);
```

Use a custom interpolator when you want complete control over how `from`/`to` are blended:

```rust
fn stepped(from: f32, to: f32, t: f32) -> f32 {
    if t < 0.5 { from } else { to }
}

let anim = Animation::new(0.0, 100.0, 500, Easing::Linear)
    .with_custom_interpolator(stepped);
```

## Manager Lifecycle and Runtime Control

`AnimationManager` now supports start/repeat/complete hooks and runtime controls:

```rust
let mut manager = AnimationManager::<8>::new();
manager.set_callbacks(AnimationManagerCallbacks {
    on_start: Some(|id| { let _ = id; }),
    on_repeat: Some(|id, iteration| { let _ = (id, iteration); }),
    on_complete: Some(|id, finished| { let _ = (id, finished); }),
});

let id = manager.start(Animation::new(0.0, 1.0, 400, Easing::InOutSine))?;
manager.set_paused(true);
manager.set_paused(false);
let _ = manager.seek(id, 200); // jump to 200ms elapsed
```

Duration introspection helpers are available on `Animation`:

```rust
let total = Animation::new(0.0, 1.0, 300, Easing::Linear)
    .with_delay(50)
    .with_repeat_mode(RepeatMode::Loop)
    .with_repeat_count(Some(3))
    .total_duration_ms(true, true); // Some(1050)
```
