
Zemeroth
========

|license|_
|loc|_
|travis-ci|_
|appveyor-ci|_
|circle-ci|_


News: https://twitter.com/ozkriff

Devlog album: http://imgur.com/a/SMVqO


Overview
--------

Zemeroth is a turn-based hexagonal tactical game written in Rust_.

.. image:: http://i.imgur.com/myVVfUW.png

.. image:: http://i.imgur.com/oXpIvb9.png

.. image:: http://i.imgur.com/ovrTxqy.gif

.. image:: http://i.imgur.com/VJaXQEJ.gif


Assets
------

Game assets are stored in a separate repo:
https://github.com/ozkriff/zemeroth_assets

Run ``git clone https://github.com/ozkriff/zemeroth_assets assets``
to download them.


Building
--------

``cargo build``.


Running
-------

``cargo run``


Android
-------

For instructions on setting up your environment see
https://github.com/tomaka/android-rs-glue#setting-up-your-environment.

Then run ``./do_android`` script.

.. image:: http://i.imgur.com/T9EgPR1.png


License
-------

Zemeroth is distributed under the terms of both
the MIT license and the Apache License (Version 2.0).

See `LICENSE-APACHE`_ and `LICENSE-MIT`_ for details.


.. |license| image:: https://img.shields.io/badge/license-MIT_or_Apache_2.0-blue.svg
.. |loc| image:: https://tokei.rs/b1/github/ozkriff/zemeroth
.. |travis-ci| image:: https://travis-ci.org/ozkriff/zemeroth.svg?branch=master
.. |appveyor-ci| image:: https://ci.appveyor.com/api/projects/status/rsxn9wh9xbpey26m/branch/master?svg=true
.. |circle-ci| image:: https://circleci.com/gh/ozkriff/zemeroth/tree/master.svg?style=svg

.. _loc: https://github.com/Aaronepower/tokei
.. _travis-ci: https://travis-ci.org/ozkriff/zemeroth
.. _appveyor-ci: https://ci.appveyor.com/project/ozkriff/zemeroth
.. _circle-ci: https://circleci.com/gh/ozkriff/zemeroth
.. _Rust: https://rust-lang.org
.. _LICENSE-MIT: LICENSE-MIT
.. _LICENSE-APACHE: LICENSE-APACHE
