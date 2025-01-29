# `bl_boot`

A mask ROM loader tool for Boufallo Lab SoCs, currently supporting:

- [x] [BL808](https://openbouffalo.github.io/chips/bl808/bootrom/)
  (_NOTE: There is no official vendor website documenting the SoC_;
  see also the [OpenBouffalo wiki](https://openbouffalo.org/index.php/BL808))

## SoMs and Boards

- [Pine64 Ox64](https://wiki.pine64.org/wiki/Ox64)
- [Sipeed M1s](https://wiki.sipeed.com/hardware/en/maix/m1s/m1s_module.html)
  * [M1s Dock](https://wiki.sipeed.com/hardware/en/maix/m1s/m1s_dock.html)
- [Sipeed MF-ST40](https://wiki.sipeed.com/hardware/zh/maixface/mfst40/mfst40.html)

## Building

Have a Rust toolchain installed with Cargo.

```sh
cargo build --release
```

## Development

This tool is written in Rust using well-known libraries for connecting via
serial, defining the CLI, and parsing/instantiating data structures:

- [serialport-rs](https://github.com/serialport/serialport-rs)
  ![serialport-rs](https://avatars.githubusercontent.com/u/32803384?s=24&v=4)
- [clap](https://docs.rs/clap)
- [zerocopy](https://docs.rs/zerocopy)
