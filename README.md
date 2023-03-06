<p align="center">
  <a href="https://swc.rs/">
    <img alt="swc" src="https://raw.githubusercontent.com/swc-project/logo/master/swc.png" width="546">
  </a>
</p>

<p align="center">
  Make the web (development) faster.
</p>

<p align="center">
  <a href="https://www.npmjs.com/package/@swc/core">
    <img alt="downloads (@swc/core)" src="https://img.shields.io/npm/dm/@swc/core?label=downloads%20%28%40swc%2Fcore%29">
  </a>
  <a href="https://www.npmjs.com/package/@swc/helpers">
    <img alt="downloads (3rd party)" src="https://img.shields.io/npm/dm/@swc/helpers?label=downloads%20%283rd%20party%29">
  </a>
</p>
<p align="center">
  <a href="https://crates.io/crates/swc_ecma_parser">
    <img alt="undefined" src="https://img.shields.io/crates/d/swc_ecma_parser.svg?label=crates.io%20downloads">
  </a>
  <a href="https://github.com/swc-project/swc/releases/latest">
    <img alt="GitHub release (latest SemVer)" src="https://img.shields.io/github/v/release/swc-project/swc">
  </a>
</p>

<p align="center">
  <img alt="GitHub code size in bytes" src="https://img.shields.io/github/languages/code-size/swc-project/swc">
  <a href="https://github.com/swc-project/swc/blob/main/package.json#L22">
    <img alt="node-current (scoped)" src="https://img.shields.io/node/v/@swc/core">
  </a>
</p>

<p align="center">
  <a href="https://discord.com/invite/GnHbXTdZz6">
    <img alt="Discord" src="https://img.shields.io/discord/889779439272075314">
  </a>
</p>

SWC (stands for `Speedy Web Compiler`) is a super-fast TypeScript / JavaScript compiler written in Rust. It's a library for Rust and JavaScript at the same time. If you are using SWC from Rust, see [rustdoc](https://rustdoc.swc.rs/swc/) and for most users, your entry point for using the library will be [parser](https://rustdoc.swc.rs/swc_ecma_parser/).

Also, SWC tries to ensure that

> If you select the latest version of each crates, it will work

for rust users.

MSRV of crates named `swc_ecma_*` is the current stable, and nightly for others.

---

If you are using SWC from JavaScript, please refer to [docs on the website](https://swc.rs/docs/installation/).

# Documentation

Check out the documentation [in the website](https://swc.rs/docs/installation/).

# Features

Please see [comparison with babel](https://swc.rs/docs/migrating-from-babel).

# Performance

Please see [benchmark results](https://swc.rs/docs/benchmark-transform) on the website.

<h2 align="center">Supporting swc</h2>

<p align="center">
  <a href="https://opencollective.com/swc">
    <img src="https://raw.githubusercontent.com/swc-project/swc-sponsor-images/main/sponsors.svg" alt="Sponsors">
  </a>
</p>

SWC is a community-driven project, and is maintained by a group of [volunteers](https://opencollective.com/swc#team). If you'd like to help support the future of the project, please consider:

-   Giving developer time on the project. (Message us on [Discord](https://discord.gg/GnHbXTdZz6) (preferred) or [Github discussions](https://github.com/swc-project/swc/discussions) for guidance!)
-   Giving funds by becoming a sponsor (see https://opencollective.com/swc)!

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). You may also find the architecture
documentation useful ([ARCHITECTURE.md](ARCHITECTURE.md)).

## License

SWC is primarily distributed under the terms of the Apache License (Version 2.0).

See [LICENSE](LICENSE) for details.
