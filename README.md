# Newsletter API

Simple newsletter service that allow users to subscribe and receive newsletter.

## setup
### pre-requisites
Linux is best to develop on, if on Windows use WSL
[Docker](https://docs.docker.com/get-docker/)
[Rust](https://www.rust-lang.org/tools/install)


### Development
- rustc 1.75.0-nightly (d627cf07c 2023-10-10) or greater
- You want to use [`cargo watch`](https://crates.io/crates/cargo-watch) for *hot reload* of files.

## Production
Digital Ocean is the *PaaS* provider, a yaml CI/CD builder is read on every commit to master, and will automatically run when CI passes.
