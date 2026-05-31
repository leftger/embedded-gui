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
