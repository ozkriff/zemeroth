# `ggwp-zgui`

Tiny and opinionated UI library for ggez game engine.

Made for Zemeroth game.
Historically was part of [Häte](https://docs.rs/hate) crate.

Limitations:

- Only provides simple labels, buttons and layouts
- Handles only basic click event
- No custom styles, only the basic one

## Examples

From simple to complicated:

- [text_button.rs](./examples/text_button.rs)
- [vertical_layout.rs](examples/vertical_layout.rs)
- [layers_layout.rs](examples/layers_layout.rs)
- [nested.rs](./examples/nested.rs)
- [remove.rs](./examples/remove.rs)
- [pixel_coordinates.rs](./examples/pixel_coordinates.rs)
- [absolute_coordinates.rs](./examples/absolute_coordinates.rs)

## What does a `ggwp-` prefix mean?

As Icefoxen asked to [not use `ggez-` prefix][ggwp]
I use `ggwp-` ("good game, well played!") to denote that the crate
belongs to ggez ecosystem, but is not official.

[ggwp]: https://github.com/ggez/ggez/issues/373#issuecomment-390461696
