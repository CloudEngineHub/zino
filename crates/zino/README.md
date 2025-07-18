[![github]](https://github.com/zino-rs/zino)
[![crates-io]](https://crates.io/crates/zino)
[![docs-rs]](https://docs.rs/zino)

[github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs

[`zino`] is a **next-generation** framework for **composable** applications in Rust
which emphasizes **simplicity**, **extensibility** and **productivity**.

## Highlights

- 🚀 Out-of-the-box features for rapid application development.
- 🎨 Minimal design, composable architecture and high-level abstractions.
- 🌐 Adopt an API-first approch to development with open standards.
- ⚡ Embrace practical conventions to get the best performance.
- 💎 Expressive ORM for MySQL, PostgreSQL and SQLite based on [`sqlx`].
- ✨ Innovations on query population, field translation and model hooks.
- 📅 Lightweight scheduler for sync and async cron jobs.
- 💠 Unified access to storage services, data sources and LLMs.
- 📊 Built-in support for [`tracing`], [`metrics`] and logging.
- 💖 Full integrations with [`actix-web`], [`axum`], [`dioxus`] and more.

## Getting started

You can start with the example [`actix-app`], [`axum-app`], [`dioxus-desktop`] or [`ntex-app`].

Here is the simplest application to run a server:
```toml
[package]
name = "zino-app"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
zino = { version = "0.37", features = ["axum"] }
```

```rust,ignore
use zino::prelude::*;

fn main() {
    zino::Cluster::boot().run()
}
```

## Feature flags

The following optional features are available:

| Name          | Description                                          | Default? |
|---------------|------------------------------------------------------|----------|
| `actix`       | Enables the integration with [`actix-web`].          | No       |
| `auth`        | Enables the authentication and authorization.        | No       |
| `axum`        | Enables the integration with [`axum`].               | No       |
| `cookie`      | Enables the support for cookies.                     | No       |
| `debug`       | Enables the features for ease of debugging.          | No       |
| `dioxus`      | Enables the integration with [`dioxus`].             | No       |
| `i18n`        | Enables the support for internationalization.        | No       |
| `inertia`     | Enables the support for the Inertia protocol.        | No       |
| `jwt`         | Enables the support for JSON Web Token.              | No       |
| `logger`      | Enables the default logger.                          | Yes      |
| `metrics`     | Enables the [`metrics`] exporter.                    | No       |
| `ntex`        | Enables the integration with [`ntex`].               | No       |
| `opa`         | Enables the support for OPA via [`regorus`].         | No       |
| `orm`         | Enables the ORM for MySQL, PostgreSQL or **SQLite**. | No       |
| `preferences` | Enables the support for application preferences.     | No       |
| `view`        | Enables the HTML template rendering.                 | No       |

[`zino`]: https://github.com/zino-rs/zino
[`sqlx`]: https://crates.io/crates/sqlx
[`tracing`]: https://crates.io/crates/tracing
[`metrics`]: https://crates.io/crates/metrics
[`regorus`]: https://crates.io/crates/regorus
[`actix-web`]: https://crates.io/crates/actix-web
[`axum`]: https://crates.io/crates/axum
[`dioxus`]: https://crates.io/crates/dioxus
[`ntex`]: https://crates.io/crates/ntex
[`actix-app`]: https://github.com/zino-rs/zino/tree/main/examples/actix-app
[`axum-app`]: https://github.com/zino-rs/zino/tree/main/examples/axum-app
[`dioxus-desktop`]: https://github.com/zino-rs/zino/tree/main/examples/dioxus-desktop
[`ntex-app`]: https://github.com/zino-rs/zino/tree/main/examples/ntex-app
