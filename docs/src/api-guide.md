# API guide

## Constructors

### `ConfigManager::for_app(app_name)`

Recommended for production desktop apps.

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?;
```

The app name must be a safe single path component.

### `ConfigManager::new()`

Convenience constructor that derives the app directory from the current
executable name.

```rust
let manager = ConfigManager::<Settings>::new();
```

This is convenient for examples and small tools, but `for_app()` is more stable
for production applications.

### `with_root_dir(path)`

Overrides the settings root directory.

```rust
let manager = ConfigManager::<Settings>::new().with_root_dir("./config");
```

Use this for tests, portable mode, app-managed storage locations, and sandboxed
hosts.

## File names

```rust
let manager = ConfigManager::<Settings>::for_app("my-app")?
    .try_with_filename("preferences.json")?;
```

`try_with_filename()` validates that the value is a plain file name.

## Loading and saving

```rust
manager.save(&settings)?;
let settings = manager.load()?;
```

`load()` expects the file to already exist.

For normal app startup:

```rust
let settings = manager.load_or_default()?;
```

## Updates

```rust
manager.update(|settings| {
    settings.launch_count += 1;
})?;
```

`update()` performs a load-or-default, applies the closure, and saves the result.
