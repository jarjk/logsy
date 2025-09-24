Logsy
=============
Just logging as it should be. While it's well under 200 lines of source code, it covers almost any practical use case you could ever encounter.

## The Looks

<!-- This section was created with the aha Ansi HTML Adapter. https://github.com/theZiz/aha -->
<pre>
<span style="color:gray;">[<span style="font-style:italic;">2025-09-23T17:55:59.557600Z </span></span><span style="font-weight:bold;"></span><span style="font-weight:bold;color:magenta;">TRACE</span> colors<span style="color:gray;">]</span> Library function called
<span style="color:gray;">[<span style="font-style:italic;">2025-09-23T17:55:59.557615Z </span></span><span style="font-weight:bold;"></span><span style="font-weight:bold;color:blue;">DEBUG</span> colors<span style="color:gray;">]</span> Auth attempt
<span style="color:gray;">[<span style="font-style:italic;">2025-09-23T17:55:59.557623Z </span></span><span style="font-weight:bold;"></span><span style="font-weight:bold;color:green;">INFO </span> colors<span style="color:gray;">]</span> Application has just started
<span style="color:gray;">[<span style="font-style:italic;">2025-09-23T17:55:59.557630Z </span></span><span style="font-weight:bold;"></span><span style="font-weight:bold;color:yellow;">WARN </span> colors<span style="color:gray;">]</span> Dereferencing null pointers harms
<span style="color:gray;">[<span style="font-style:italic;">2025-09-23T17:55:59.557637Z </span></span><span style="font-weight:bold;"></span><span style="font-weight:bold;color:red;">ERROR</span> colors<span style="color:gray;">]</span> This application got a boo-boo and going to be terminated
</pre>

## Common concepts

* Use `logsy::to_console()` to start logging to `stderr`
* Use `logsy::to_file(path, append: bool)` to start logging into a specified file. If parent dir doesn't exist, it will be created
* Use `logsy::set_level(level)` to alter logging level (defaults to `LevelFilter::Info`)
* You can combine `logsy::to_console()` and `logsy::to_file(path, append: bool)` in the same application
* Rather than being a standalone package, it's simply a backend for the ubiquitous [log](https://crates.io/crates/log) crate

## Features

<div class="warning">

Currently all features are enabled by default.\
To disable them, use `cargo add logsy --no-default-features`,
or add eg.: `logsy = { version = "1.0.1", default-features = false, features = ["env"] }` to your `Cargo.toml` file.

</div>

### `env`

Read the `RUST_LOG` env var and set it's value as default log level.

### `file`

Enables writing log to file: `logsy::to_file(path, append: bool)`.

### `time`

Uses `humantime` to add UTC RFC3339 timestamp to each record.

### `styled`

Stylize console output.

## Example

```rust
logsy::to_console();
logsy::to_file("logs/main.log", true); // true: open in append mode

log::info!("Application has just started");
log::warn!("Dereferencing null pointers harms");
log::error!("This application got a boo-boo and going to be terminated");
```
