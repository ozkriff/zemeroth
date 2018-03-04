# Zemeroth [![][img_license]](#license) [![][img_loc]][loc]

[img_license]: https://img.shields.io/badge/License-MIT_or_Apache_2.0-blue.svg
[img_loc]: https://tokei.rs/b1/github/ozkriff/zemeroth

Zemeroth is a turn-based hexagonal tactical game written in [Rust].

[Rust]: https://www.rust-lang.org

![](https://i.imgur.com/uOwrmIV.png)

**News**: [@ozkriff on twitter](https://twitter.com/ozkriff) |
[ozkriff.github.io](https://ozkriff.github.io) |
[devlog on imgur](https://imgur.com/a/SMVqO)

**Status**:
[![][img_travis-ci]][travis-ci]
[![][img_appveyor-ci]][appveyor-ci]
[![][img_circle-ci]][circle-ci]
[![dependency status][img_deps-rs]][deps-rs]

[img_travis-ci]: https://img.shields.io/travis/ozkriff/zemeroth/master.svg?label=Linux|OSX
[img_appveyor-ci]: https://img.shields.io/appveyor/ci/ozkriff/zemeroth.svg?label=Windows
[img_circle-ci]: https://img.shields.io/circleci/project/github/ozkriff/zemeroth/master.svg?label=Android
[img_deps-rs]: https://deps.rs/repo/github/ozkriff/zemeroth/status.svg

[loc]: https://github.com/Aaronepower/tokei
[travis-ci]: https://travis-ci.org/ozkriff/zemeroth
[appveyor-ci]: https://ci.appveyor.com/project/ozkriff/zemeroth
[circle-ci]: https://circleci.com/gh/ozkriff/zemeroth
[deps-rs]: https://deps.rs/repo/github/ozkriff/zemeroth


## Precompiled Binaries

Precompiled binaries for Linux, Windows, OS X and Android:
<https://github.com/ozkriff/zemeroth/releases>


## Screenshot

![](https://i.imgur.com/wMG3KkA.png)


## Gifs

![](https://i.imgur.com/R298zUm.gif)

![](https://i.imgur.com/mTTrWHu.gif)
![](https://i.imgur.com/2dR278L.gif)
![](https://i.imgur.com/wZZdlXs.gif)
![](https://i.imgur.com/1shTV2q.gif)


## Videos

<https://www.youtube.com/user/ozkriff619/videos>


## Building from Source

```bash
# Clone this repo
git clone https://github.com/ozkriff/zemeroth
cd zemeroth

# Assets are stored in a separate repo.
# Zemeroth expects them to be in `assets` directory.
git clone https://github.com/ozkriff/zemeroth_assets assets

# Compile a debug version
cargo build

# Run it
cargo run
```


## Building from Source for Android

[Set up a build environment][android setup] and run `./do_android` script:

![android screenshot](https://i.imgur.com/T9EgPR1.png)

[android setup]: https://github.com/tomaka/android-rs-glue#setting-up-your-environment


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
