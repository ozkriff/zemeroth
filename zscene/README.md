# `zscene`

`zscene` is a simple scene and declarative animation manager.

Made for Zemeroth game.
Historically was part of [Häte](https://docs.rs/hate) crate.

This crate provides:

- `Sprite`s that can be shared
- `Scene` and `Action`s to manipulate it
- Basic layers

## Examples

The following code sample shows how to create sprite,
add it to the scene and move it:

```rust
let mut sprite = Sprite::from_path(context, "/fire.png", 0.5)?;
sprite.set_pos(Vec2::new(0.0, -1.0));
let delta = Vector2::new(0.0, 1.5);
let time = Duration::from_millis(2_000);
let action = action::Sequence::new(vec![
    action::Show::new(&self.layers.fg, &sprite).boxed(), // show the sprite
    action::MoveBy::new(&sprite, delta, time).boxed(), // move it
]);
self.scene.add_action(action.boxed());
```

See [examples/action.rs](./examples/action.rs) for a complete example.
