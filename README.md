# Pass Search Provider for GNOME Shell

This is a GNOME search provider for use with
[pass](https://www.passwordstore.org/), the standard Unix password manager.

It is written 100% in rust using the [ripasso](https://github.com/cortex/ripasso) library and has no runtime dependencies, not even pass itself.


## Installation

### Manual
Clone this repository and run:

 ```bash
 cargo build --release && sudo ./install.sh
 ```

Make sure the search provider is enabled in GNOME settings under Search.

## Setting a custom PASSWORD_STORE_DIR

`systemctl --user edit io.m51.Pass.SearchProvider.service`

Add the following to the file:

```ini
[Service]
Environment="PASSWORD_STORE_DIR=/path/to/password-store"
```
