# Biscuit

A sleek, native multi-tab Markdown and LaTeX editor powered by GTK4 and Libadwaita.

## Features
- **Multi-Tab Interface:** Seamlessly edit multiple Markdown and LaTeX documents concurrently.
- **Native GTK4 Integration:** Designed strictly around Libadwaita for a beautiful, system-native aesthetic.
- **Dynamic Previews:** Toggleable preview panes that live alongside your source code.

## Installation

### Debian / Ubuntu
You can download the pre-compiled `.deb` package directly from the [GitHub Releases](../../releases) page!

```bash
sudo dpkg -i biscuit_0.1.0_amd64.deb
sudo apt-get install -f # To resolve any missing dependencies
```

### Build from Source
If you prefer to compile from source, ensure you have Rust and the required GTK4/Libadwaita development libraries installed.

```bash
sudo apt-get install libgtk-4-dev libadwaita-1-dev
cargo build --release
```

## License
MIT License. Maintained by 7CGPA-Labs and GaganCJ.
