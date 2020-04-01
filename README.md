# ![title image](zemeroth.svg)

[![mit license][img_license]](#license) [![line count][img_loc]][loc]

[img_license]: https://img.shields.io/badge/License-MIT_or_Apache_2.0-blue.svg
[img_loc]: https://tokei.rs/b1/github/ozkriff/zemeroth

Zemeroth is a turn-based hexagonal tactical game written in [Rust].

[Rust]: https://www.rust-lang.org

**Support**: [**patreon.com/ozkriff**](https://patreon.com/ozkriff)

**News**: [@ozkriff on twitter](https://twitter.com/ozkriff) |
[ozkriff.games](https://ozkriff.games) |
[facebook](https://fb.com/ozkriff.games) |
[devlog on imgur](https://imgur.com/a/SMVqO)

**Status**:
[![Github Actions][img_gh-actions]][gh-actions]

[img_gh-actions]: https://github.com/ozkriff/zemeroth/workflows/CI/badge.svg

[loc]: https://github.com/Aaronepower/tokei
[gh-actions]: https://github.com/ozkriff/zemeroth/actions?query=workflow%3ACI

## Online Version

You can play an online WebAssembly version of Zemeroth at
[ozkriff.itch.io/zemeroth](https://ozkriff.itch.io/zemeroth)

## Precompiled Binaries

Precompiled binaries for Linux, Windows and macOS:
[github.com/ozkriff/zemeroth/releases](https://github.com/ozkriff/zemeroth/releases)

## Screenshots

!["big" screenshot](https://i.imgur.com/iUkyqIq.png)

!["campaign" screenshot](https://i.imgur.com/rQkH5y2.png)

![web version of a phone](https://i.imgur.com/cviYFkY.jpg)

## Gifs

![main gameplay animation](https://i.imgur.com/HqgHmOH.gif)

## Videos

[youtube.com/c/andreylesnikov/videos](https://youtube.com/c/andreylesnikov/videos)

## Vision

[The initial vision](https://ozkriff.github.io/2017-08-17--devlog/index.html#zemeroth)
of the project is:

- Random-based skirmish-level digital tabletop game;
- Single player only;
- 3-6 fighters under player’s control;
- Small unscrollable maps;
- Relatively short game session (under an hour);
- Simple vector 2d graphics with just 3-5 sprites per unit;
- Reaction attacks and action’s interruption;
- Highly dynamic (lots of small unit moves as a side effect of other events);
- Intentionally stupid and predictable AI;

## Roadmap

- [ ] Phase One: Linear Campaign Mode

  An extended prototype focused just on tactical battles.

  - [x] [v0.4](https://github.com/ozkriff/zemeroth/projects/1)
    - [x] Basic gameplay with reaction attacks
    - [x] Minimal text-based GUI
    - [x] Basic agent abilities: jumps, bombs, dashes, etc
  - [x] [v0.5](https://github.com/ozkriff/zemeroth/projects/2)
    - [x] Basic campaign mode
    - [x] Armor and Break stats ([#70](https://github.com/ozkriff/zemeroth/issues/70))
    - [x] Dynamic blood splatters ([#86](https://github.com/ozkriff/zemeroth/issues/86))
    - [x] Web version
    - [x] Tests
    - [x] Hit chances
  - [x] [v0.6](https://github.com/ozkriff/zemeroth/projects/3)
    - [x] Agent upgrades ([#399](https://github.com/ozkriff/zemeroth/issues/399))
    - [x] Flip agent sprites horizontally when needed ([#115](https://github.com/ozkriff/zemeroth/issues/115))
    - [x] Multiple sprites per agent type ([#114](https://github.com/ozkriff/zemeroth/issues/114))
  - [ ] GUI icons ([#276](https://github.com/ozkriff/zemeroth/issues/276))
  - [ ] Sound & Music ([#221](https://github.com/ozkriff/zemeroth/issues/221))
  - [ ] Reduce text overlapping ([#214](https://github.com/ozkriff/zemeroth/issues/214))
  - [ ] Move back after a successful dodge ([#117](https://github.com/ozkriff/zemeroth/issues/117))
  - [ ] Easing ([#26](https://github.com/ozkriff/zemeroth/issues/26))
  - [ ] Path selection ([#280](https://github.com/ozkriff/zemeroth/issues/280),
    [#219](https://github.com/ozkriff/zemeroth/issues/219))
  - [ ] Intermediate bosses
  - [ ] Main boss
  - [ ] Neutral agents ([#393](https://github.com/ozkriff/zemeroth/issues/393))
  - [ ] Weight component ([#291](https://github.com/ozkriff/zemeroth/issues/291))
  - [ ] Basic inventory system: slots for artifacts
  - [ ] Ranged units
  - [ ] More agent types
  - [ ] More passive abilities that allow agents to make actions
    during enemy's turn
    ([#354](https://github.com/ozkriff/zemeroth/issues/354))
  - [ ] More complex multieffect abilities/actions
  - [ ] Guide ([#451](https://github.com/ozkriff/zemeroth/issues/451))
  - [ ] Save/load ([#28](https://github.com/ozkriff/zemeroth/issues/28))
  - [ ] Android version

- [ ] Phase Two: Strategy Mode

  A not-so-linear strategic layer will be added on top of tactical battles.
  Simple non-linear story and meta-gameplay.

  - [ ] Global map
  - [ ] Dialog system
  - [ ] Quest system
  - [ ] NPC/Agent/Masters system

## Inspiration

Tactical battle mechanics are mostly inspired by these games:

- [ENYO](https://play.google.com/store/apps/details?id=com.tinytouchtales.enyo)
- [Hoplite](https://play.google.com/store/apps/details?id=com.magmafortress.hoplite)
- [Into the Breach](https://store.steampowered.com/app/590380/Into_the_Breach)
- [Banner Saga](https://store.steampowered.com/app/237990/The_Banner_Saga)
  (Survival Mode)
- [Auro](https://store.steampowered.com/app/459680/Auro_A_MonsterBumping_Adventure)
- [Minos Strategos](https://store.steampowered.com/app/577490/Minos_Strategos)
- [Battle Brothers](https://store.steampowered.com/app/365360/Battle_Brothers)

## Building from Source

Install all [miniquad's system dependencies][mq_sys_deps].

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

[mq_sys_deps]: https://github.com/not-fl3/miniquad/tree/faabb54d3#building-examples

## WebAssembly

```bash
rustup target add wasm32-unknown-unknown
./utils/wasm/build.sh
cargo install basic-http-server
basic-http-server static
```

Then open `http://localhost:4000` in your browser.

## Dependencies

The key external dependency of Zemeroth is [good-web-game]/[miniquad]
(see [#564 "Migrate to miniquad"]).

This repo contains a bunch of helper crates:

- [zcomponents] is a simple component storage
- [zgui] is a simple and opinionated GUI library
- [zscene] is a simple scene and declarative animation manager

[good-web-game]: https://github.com/not-fl3/good-web-game
[miniquad]: https://github.com/not-fl3/miniquad
[#564 "Migrate to miniquad"]: https://github.com/ozkriff/zemeroth/issues/564
[zcomponents]: ./zcomponents
[zscene]: zscene
[zgui]: zgui

## Contribute

If you want to help take a look at issues with `help-wanted` label attached:

[github.com/ozkriff/zemeroth/labels/help-wanted](https://github.com/ozkriff/zemeroth/labels/help-wanted)

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

## License

Zemeroth is distributed under the terms of both
the MIT license and the Apache License (Version 2.0).
See [LICENSE-APACHE] and [LICENSE-MIT] for details.

Zemeroth's text logo is based on the "Old London" font
by [Dieter Steffmann](http://www.steffmann.de).

[LICENSE-MIT]: LICENSE-MIT
[LICENSE-APACHE]: LICENSE-APACHE
