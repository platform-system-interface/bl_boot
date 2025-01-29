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

## Running

To run a given flat binary `c906.bin` on the D0 aka MM (C906) core:

```sh
cargo run --release -- --d0-binary c906.bin
```

For more options, see the help:

```sh
cargo run --release -- -h
```

## Development

This tool is written in Rust :crab: using well-known libraries from the Rust
community for connecting via serial, defining the CLI, and parsing/instantiating
data structures, including:

- ![serialport-rs](https://avatars.githubusercontent.com/u/32803384?s=24&v=4)
  [serialport-rs](https://github.com/serialport/serialport-rs)
- ![clap](https://avatars.githubusercontent.com/u/39927937?s=24&v=4)
  [clap](https://docs.rs/clap)
- [zerocopy](https://docs.rs/zerocopy)
- [bitfield-struct](https://docs.rs/bitfield-struct)

## History

Based on the [vendor SDK](https://github.com/bouffalolab/bouffalo_sdk), their
[bflb-mcu-tool](https://github.com/openbouffalo/bflb-mcu-tool) and [smaeul's
bouffalo-loader](https://github.com/smaeul/bouffalo-loader) as well as the
[documentation of fields and structs in the OpenBouffalo project](
https://github.com/openbouffalo/bouffalo_structs/tree/main/bl808), we were able
to create a rather comprehensive tool that is simple to use. Big thanks go out
to everyone in the community who helped us out and shared their findings.
