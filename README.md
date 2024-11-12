[![Build Status](https://github.com/bmatcuk/libuv-rs/actions/workflows/test-and-release.yml/badge.svg)](https://github.com/bmatcuk/libuv-rs/actions)
[![Latest Release](https://img.shields.io/crates/v/libuv)](https://crates.io/crates/libuv)
[![Documentation](https://docs.rs/libuv/badge.svg)](https://docs.rs/libuv)

# libuv-rs
A safe rust wrapper for [libuv].

## Getting Started
Include [libuv-rs] as a dependency in your Cargo.toml:

```toml
[dependencies]
libuv = "~1.0.0"
```

[libuv-rs] uses semantic versioning.

As of v2.0.1, libuv-rs supports the `skip-pkg-config` feature. This is passed
to [libuv-sys2] to skip searching for a local install of [libuv] via pkg-config
and, instead, causes [libuv-sys2] to build [libuv] from source.

You'll want to make sure to familiarize yourself with [libuv] by reading
[libuv's documentation]. You can then familiarize yourself with [libuv-rs] by
reading the [examples] and [documentation].

## Unimplemented
[libuv-rs] strives to implement wrappers for all [libuv] functionality.
However, some functionality was purposefully excluded as rust provides
implementations of its own. That is: threads and synchronization (mutexes,
locks, semaphores, conditional variables, barriers, etc).

If your rust project would benefit from [libuv]'s threading or synchronization
primitives, please file an Issue on github and I'll implement wrappers for it!

## Cross-Platform Considerations
[libuv-rs] depends on [libuv-sys2], which depends on [bindgen]. On Windows,
[bindgen] requires rust's msvc toolchain.

[bindgen]: https://rust-lang.github.io/rust-bindgen/
[documentation]: https://docs.rs/libuv
[examples]: https://github.com/bmatcuk/libuv-rs/tree/master/examples
[libuv's documentation]: http://docs.libuv.org
[libuv-rs]: https://github.com/bmatcuk/libuv-rs/
[libuv-sys2]: https://github.com/bmatcuk/libuv-sys/
[libuv]: https://libuv.org/
