# bootrust

**An elegant, macro-free Data Access Layer abstraction — a simple and easy-to-use ORM powered by Serde.**

---

## Features

* **Macro-Free Design**: No need to define any macros or annotations for entities, keeping your code clean and simple.
* **Serde Support**: Leverages Serde for seamless serialization and deserialization.
* **Dependency Injection**: Switch database backends simply by changing dependencies in `Cargo.toml`, with no changes needed in business logic.
* **Flexible Extension**: Customizable `entity_to_map` and `row_to_entity` functions to handle special type mappings.

---

## Installation

Add the following to your project's `Cargo.toml`:

```toml
[dependencies]
bootrust = { version = "0.1", features = ["sqlite_async"] }  # Core BOOTrust library with async sqlite support 
```

---

## Quick Start

```sh
# Run the simple example with SQLite backend
cargo run --example simple_example --features=sqlite_async
```

---

## Type Mapping

| Rust Type       | Example DB Types                                                      | Serde Conversion                               | Notes                                                                                                                                            |
| --------------- | --------------------------------------------------------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `String`        | PostgreSQL: `TEXT`<br>MySQL: `TEXT`<br>SQLite: `TEXT`                 | Built-in | Text type                                                                                                                                        |
| `i32`           | PostgreSQL: `INTEGER`<br>MySQL: `INT`<br>SQLite: `INTEGER`            | Built-in | 32-bit integer                                                                                                                                   |
| `i64`           | PostgreSQL: `BIGINT`<br>MySQL: `BIGINT`<br>SQLite: `INTEGER`          | Built-in                                           | 64-bit integer; all SQLite integers are stored as `INTEGER`                                                                                      |
| `Vec<T>`        | PostgreSQL: `BYTEA`<br>MySQL: `BLOB`<br>SQLite: `BLOB`                | Built-in                                           | Binary data                                                                                                                                      |
| `Option<T>`     | Same as type `T`, but allows `NULL`                                   | Built-in                                           | Optional value                                                                                                                                   |
| `DateTime<Utc>` | PostgreSQL: `BIGINT`<br>MySQL: `BIGINT`<br>SQLite: `TEXT` / `INTEGER` | `#[serde(with = "chrono::serde::ts_seconds")]` | - MySQL: only supports integer (Unix timestamp) format.<br>- PostgreSQL & SQLite: supports both ISO-8601 text and integer with Serde attributes. |

> **Notes**:
>
> 1. Due to Rust's orphan rule, this crate cannot provide a custom Serde implementation for `chrono::DateTime`. If you cannot change the database column type (e.g., for existing tables), you must manually define the mapping between `DateTime<Utc>` and `Value::DateTime` in `entity_to_map` / `row_to_entity`.
> 2. The MySQL driver does not support serializing `DateTime<Utc>` as `TEXT`; it only supports integer mappings.
> 3. PostgreSQL has limited support for null types, only allowing empty values for `TEXT`.

---

## Switching Database

Just swap the feature flag in Cargo.toml to switch database backends.

Note: Ensure that your runtime and async driver dependencies are compatible.
```toml
[dependencies]
bootrust = { version = "0.1", features = ["postgresql_async"] }
# bootrust = { version = "0.1", features = ["mysql_async"] }
```

No changes to your business logic are needed—just rebuild with `cargo build`.

---

## Docs & Contributing

* Online Docs: [https://docs.rs/bootrust](https://docs.rs/bootrust)
* Issues & PRs: [https://github.com/tianzeshi-study/bootrust](https://github.com/tianzeshi-study/bootrust)

---

## License

This project is licensed under the [MIT](./LICENSE) license.
