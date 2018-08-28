# Zemeroth [![mit license][img_license]](#license) [![line count][img_loc]][loc]

[img_license]: https://img.shields.io/badge/License-MIT_or_Apache_2.0-blue.svg
[img_loc]: https://tokei.rs/b1/github/ozkriff/zemeroth

Zemeroth is a turn-based hexagonal tactical game written in [Rust].

[Rust]: https://www.rust-lang.org

![tiny screenshot (I don't have any logo yet)](https://i.imgur.com/uOwrmIV.png)

**News**: [@ozkriff on twitter](https://twitter.com/ozkriff) |
[ozkriff.github.io](https://ozkriff.github.io) |
[devlog on imgur](https://imgur.com/a/SMVqO)

**Status**:
[![travis status][img_travis-ci]][travis-ci]
[![appveyor status][img_appveyor-ci]][appveyor-ci]
[![dependency status][img_deps-rs]][deps-rs]

[img_travis-ci]: https://img.shields.io/travis/ozkriff/zemeroth/master.svg?label=Linux|OSX
[img_appveyor-ci]: https://img.shields.io/appveyor/ci/ozkriff/zemeroth/master.svg?label=Windows
[img_deps-rs]: https://deps.rs/repo/github/ozkriff/zemeroth/status.svg

[loc]: https://github.com/Aaronepower/tokei
[travis-ci]: https://travis-ci.org/ozkriff/zemeroth
[appveyor-ci]: https://ci.appveyor.com/project/ozkriff/zemeroth
[deps-rs]: https://deps.rs/repo/github/ozkriff/zemeroth

## Precompiled Binaries

Precompiled binaries for Linux, Windows and macOS:
<https://github.com/ozkriff/zemeroth/releases>

## Screenshot

!["big" screenshot](https://i.imgur.com/wMG3KkA.png)

## Gifs

![main gameplay animation](https://i.imgur.com/R298zUm.gif)

![gif: hammerman in action](https://i.imgur.com/mTTrWHu.gif)
![gif: jump ability](https://i.imgur.com/2dR278L.gif)
![gif: fire bomb ability](https://i.imgur.com/wZZdlXs.gif)
![gif: imp summoner in action](https://i.imgur.com/1shTV2q.gif)

## Videos

<https://www.youtube.com/user/ozkriff619/videos>

## Building from Source

As Zemeroth uses [ggez] game engine,
you need to have the SDL2 libraries installed on your system.
The best way to do this is documented [by the SDL2 crate][install sdl2].

[ggez]: https://github.com/ggez/ggez
[install sdl2]: https://github.com/AngryLawyer/rust-sdl2#user-content-requirements

```bash
# Clone this repo
git clone https://github.com/ozkriff/zemeroth
cd zemeroth

# Assets are stored in a separate repo.
# Zemeroth expects them to be in `assets` directory.
git clone https://github.com/ozkriff/zemeroth_assets assets

# Compile a release version (debug builds give low FPS at the moment)
cargo build --release

# Run it
cargo run --release
```

## Dependencies

The key external dependency of Zemeroth is [ggez] game engine.

This repo contains a bunch of helper crates:

- [zcomponents] is a simple component storage
- [ggwp-zgui] is a simple and opinionated ggez-based GUI library
- [ggwp-zscene] is a simple scene and declarative animation manager

[zcomponents]: ./zcomponents
[ggwp-zscene]: ./ggwp-zscene
[ggwp-zgui]: ./ggwp-zgui

## Contribute

If you want to help take a look at issues with `help-wanted` label attached:

<https://github.com/ozkriff/zemeroth/labels/help-wanted>

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

## License

Zemeroth is distributed under the terms of both
the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE] and [LICENSE-MIT] for details.

[LICENSE-MIT]: LICENSE-MIT
[LICENSE-APACHE]: LICENSE-APACHE
