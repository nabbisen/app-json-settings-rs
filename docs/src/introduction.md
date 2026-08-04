# Introduction

`app-json-settings` is a tiny Rust crate for storing one typed settings value as
JSON.

The crate is intentionally narrow:

* it stores a complete Serde-serializable value;
* it uses one JSON file;
* it provides first-run default creation;
* it provides a small read-modify-write helper;
* it lets applications choose the settings root directory when needed.

It is best suited to desktop GUI apps, small local tools, examples, and apps
that want a simple typed settings layer without a larger configuration system.

## Non-goals

The crate is not:

* a database;
* a dynamic key-value preference service;
* a multi-file configuration framework;
* a schema migration engine;
* a general Windows storage abstraction.

It does not take a logging or tracing dependency, either. Failures are
reported through `Result`, not logged — so a caller who wants to know that a
silent fallback fired must ask, via `try_new()` or `for_app()`, rather than
expect a warning to appear on its own.

When the settings model becomes large or needs migrations, prefer to keep the
migration policy in the application layer and use this crate only for the
persistence primitive.
