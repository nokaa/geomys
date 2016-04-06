# Geomys [![Build Status](https://travis-ci.org/nokaa/geomys.svg?branch=master)](https://travis-ci.org/nokaa/geomys)

An async gopher server written in Rust using rotor. The number of visits to
the server can be viewed at `/visits`.

By default, Geomys runs at `0.0.0.0:70` and reads from `/var/gopher`. This
can be overridden in several ways. In order of precedence, passing CLI args
holds greatest precedence, followed by the file `~/.config/geomys/config`. If
neither of these values are present, `/etc/geomys/config` is checked. If none
of these values are given, the default is used. An example configuration file
is available in the repository as `config`.

### Build

Geomys requires the Rust toolchain to be installed on your system. It is
tested with the current stable, beta, and nightly compilers.

```
cargo build --release
sudo ./target/release/geomys
```

##### Building on ARM

mio v0.5.0, a dependency of rotor, does not currently build on ARM. I have
patched it so that it will build properly without breaking anything.
In order to use the patched version of mio:

```
git clone https://git.nokaa.moe/nokaa/mio_arm /my/path/mio
cd /my/path/mio
git checkout v0.5.1
cd /path/to/geomys
mkdir .cargo
echo 'paths = ["/my/path/mio",]' > .cargo/config
cargo build --release
sudo ./target/release/geomys
```
