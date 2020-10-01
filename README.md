fedora-container-release
========================

This project is a Command Line Interface (CLI) tool which fetch the Fedora container base image
rootfs from [Fedora's build system](https://koji.fedoraproject.org) and prepares it to be pushed to
[docker-brew-fedora](https://github.com/fedora-cloud/docker-brew-fedora) repository which is used to
update our images in the Docker Hub.

This cli is currently used in a [GitHub action](https://github.com/fedora-cloud/docker-brew-fedora/blob/master/.github/workflows/main.yml) define in the docker-brew-fedora repo


## Development

To start hacking on that repository you need the Rust package manager Cargo installed on your local machine.

## Clone the repository

```
$ git clone https://github.com/fedora-cloud/fedora-container-release.git
$ cd fedora-container-release
```

## Run the application

```
$ cargo run -- --release 33
```

## Build a release binary

```
$ cargo build --release
$ ll target/release/fedora-container-release
$ target/release/fedora-container-release --help
fedora-container-release 0.1.0

USAGE:
    fedora-container-release --release <release>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -r, --release <release>
```

