Logsy
=============
Just logging as it should be. While it barely exceeds 100 lines of source code, it covers almost any practical case you could ever encounter.

## Common concepts

* Use ``logsy::set_echo()`` to start logging into stdout
* Use ``logsy::set_filename()`` to start logging into a specified file. If parent dir doesn't exists, it's going to be created
* Use ``logsy::set_level()`` to alter logging level (defaults to ``LevelFilter::Info``)
* You can combine ``logsy::set_echo()`` and ``logsy::set_filename()`` in the same application
* You can alter the settings by calling ``logsy::set_echo()`` or ``logsy::set_filename()`` again at any time
* Rather being a standalone package, it's simply a backend for the famous [log](https://crates.io/crates/log) crate

## Example
```rust
use log::*;

fn main() {
    logsy::set_echo(true);
    logsy::set_filename(Some("logs/main.log")).expect("Couldn't open main.log");

    info!("Application has just started");
    warn!("Dereferencing null pointers harms");
    error!("This application got a boo-boo and going to be terminated");
}
```
