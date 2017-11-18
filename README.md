
# Zemeroth

[![][img_license]][LICENSE-MIT]
[![][img_loc]][loc]
[![][img_travis-ci]][travis-ci]
[![][img_appveyor-ci]][appveyor-ci]
[![][img_circle-ci]][circle-ci]

[img_license]: https://img.shields.io/badge/License-MIT_or_Apache_2.0-blue.svg
[img_loc]: https://tokei.rs/b1/github/ozkriff/zemeroth
[img_travis-ci]: https://img.shields.io/travis/ozkriff/zemeroth/master.svg?label=Linux|OSX
[img_appveyor-ci]: https://img.shields.io/appveyor/ci/ozkriff/zemeroth.svg?label=Windows
[img_circle-ci]: https://img.shields.io/circleci/project/github/ozkriff/zemeroth/master.svg?label=Android

[loc]: https://github.com/Aaronepower/tokei
[travis-ci]: https://travis-ci.org/ozkriff/zemeroth
[appveyor-ci]: https://ci.appveyor.com/project/ozkriff/zemeroth
[circle-ci]: https://circleci.com/gh/ozkriff/zemeroth


News: <https://twitter.com/ozkriff>

Devlog album: <http://imgur.com/a/SMVqO>


## Downloads

Precompiled binaries for Linux, Windows, OS X and Android:
<https://github.com/ozkriff/zemeroth/releases>


## Overview

Zemeroth is a turn-based hexagonal tactical game written in [Rust].

![](https://i.imgur.com/myVVfUW.png)

![](https://i.imgur.com/oXpIvb9.png)

![](https://i.imgur.com/ovrTxqy.gif)

![](https://i.imgur.com/VJaXQEJ.gif)

[Rust]: https://rust-lang.org



## Assets

Game assets are stored in a separate repo:
https://github.com/ozkriff/zemeroth_assets

Run `git clone https://github.com/ozkriff/zemeroth_assets assets`
to download them.


## Building

`cargo build`.


## Running

`cargo run`.


## Android

For instructions on setting up your environment see
<https://github.com/tomaka/android-rs-glue#setting-up-your-environment>.

Then run `./do_android` script:

![android screenshot](https://i.imgur.com/T9EgPR1.png)


## License

Zemeroth is distributed under the terms of both
the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE] and [LICENSE-MIT] for details.

[LICENSE-MIT]: LICENSE-MIT
[LICENSE-APACHE]: LICENSE-APACHE
