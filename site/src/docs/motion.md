# Motion

Mok's motion lives in `mok_gpui::motion`: four durations, a few easings, and components that move. Every one honors the theme's reduced motion.

## Durations and easings

| Item | Value |
| --- | --- |
| `FAST`, `BASE`, `SLOW` | 120, 200 and 320 ms. |
| `THEME` | 280 ms, the theme's cross-fade. |
| `NUDGE` | 4px, the unit a small entrance travels by. |
| `duration(base, cx)` | `base`, or 1 ms under reduced motion. Pass every duration through it. |
| `ease_out_cubic`, `ease_in_out_cubic`, `lerp` | Plain functions of `t` from 0 to 1. |
| `spring` | A damped spring. It overshoots past 1 before it settles. |

GPUI hands an easing's value to the animator unclamped. Use `spring` inside the animator, on what may overshoot, such as a thumb's travel; its colors keep a plain easing.

## Reduced motion

The theme's `reduced_motion` field starts off. Mok does not read the system's setting; the app sets it:

```rust
Theme::update(cx, |theme| theme.reduced_motion = true);
```

Under it, `duration` returns 1 ms, so a change lands at once. A motion that repeats, such as a spinner or a breathing dot, holds still instead: at 1 ms a turn it would flicker. Mok's spinner checks the flag itself; an app's own repeating animation checks it too, as the example's dot does.

## Components that move

| Component | Moves |
| --- | --- |
| `Transition::new(id, shown)` | Its children in and out as `shown` turns: `Entrance::Fade`, `Rise` or `Slide`. |
| `Stagger` | Rows in one after another. |
| `Flash::new(id, key)` | A tint each time `key` changes, fading out. |
| `Shake::new(id, key)` | A shake each time `key` changes; still under reduced motion. |
| `Spinner`, `ProgressBar`, `ProgressRing` | Progress, held still under reduced motion where they repeat. |
| `Skeleton`, `Shimmer` | A placeholder while content loads, and a band that sweeps over what it holds. |
| `Flip`, `AnimatePresence`, `Reorder` | Keyed rows that glide to new places, enter, leave, or drag. |

`Flash` and `Shake` read the time since `key` changed; they do not restart an animation, so what they hold keeps its focus and state.

## A window that moves

The example shows a panel that rises in, a count that flashes, a spinner, and a breathing dot, with a button that turns reduced motion on and off.

```rust example=motion
```
