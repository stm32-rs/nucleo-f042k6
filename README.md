nucleo-f042k6
=============

_nucleo-f042k6_ contains a basic board support packet for the fabulous [STM
Nucleo-F042K6][] microcontroller board to write firmwares using Rust. This
standard format board 1 user programmable LED as well as "Arduino Nano" headers
which can be used to connect peripherals. It also contains a (non-removable,
unlike the bigger Nucleo boards) capable ST-Link V2 debugging interface, so all
that one needs to get going with progamming this device is:

* A STM Nucleo-F042K6 board
* A computer (macOS and Linux work perfectly, Windows should work but was not tested)
* A bit of open source software

[STM Nucleo-F042K6]: https://os.mbed.com/platforms/ST-Nucleo-F042K6/
[cortex-m]:(https://github.com/japaric/cortex-m)
[cortex-m-rt]:(https://github.com/japaric/cortex-m-rt)

License
-------

[0-clause BSD license](LICENSE-0BSD.txt).
