# REVM library for javascript

It is currently very restricted but it is working and it is good example of usage of revm as js lib.

Dev problems:
* auto_impl still uses std and it cant be compiles out of box: https://github.com/bluealloy/revm/issues/4
* windows build wth `cc` does not pass and it will not be builded: https://github.com/bluealloy/revm/issues/3