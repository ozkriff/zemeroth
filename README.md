# Zemeroth [![][img_license]](#license) [![][img_loc]][loc]

[img_license]: https://img.shields.io/badge/License-MIT_or_Apache_2.0-blue.svg
[img_loc]: https://tokei.rs/b1/github/ozkriff/zemeroth

Zemeroth is a turn-based hexagonal tactical game written in [Rust].

[Rust]: https://rust-lang.org

![](https://i.imgur.com/uOwrmIV.png)

**News**: [@ozkriff on twitter](https://twitter.com/ozkriff) |
[ozkriff.github.io](https://ozkriff.github.io) |
[devlog on imgur](https://imgur.com/a/SMVqO)

**Status**:
[![][img_travis-ci]][travis-ci]
[![][img_appveyor-ci]][appveyor-ci]
[![][img_circle-ci]][circle-ci]

[img_travis-ci]: https://img.shields.io/travis/ozkriff/zemeroth/master.svg?label=Linux|OSX
[img_appveyor-ci]: https://img.shields.io/appveyor/ci/ozkriff/zemeroth.svg?label=Windows
[img_circle-ci]: https://img.shields.io/circleci/project/github/ozkriff/zemeroth/master.svg?label=Android

[loc]: https://github.com/Aaronepower/tokei
[travis-ci]: https://travis-ci.org/ozkriff/zemeroth
[appveyor-ci]: https://ci.appveyor.com/project/ozkriff/zemeroth
[circle-ci]: https://circleci.com/gh/ozkriff/zemeroth


## Precompiled Binaries

Precompiled binaries for Linux, Windows, OS X and Android:
<https://github.com/ozkriff/zemeroth/releases>


## Screenshots

![](https://i.imgur.com/myVVfUW.png)

![](https://i.imgur.com/oXpIvb9.png)


## Gifs

![](https://i.imgur.com/ovrTxqy.gif)

![](https://i.imgur.com/VJaXQEJ.gif)


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


## License

Zemeroth is distributed under the terms of both
the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE] and [LICENSE-MIT] for details.

[LICENSE-MIT]: LICENSE-MIT
[LICENSE-APACHE]: LICENSE-APACHE
