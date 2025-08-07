# wait-for-file

A GTK4 application that waits for a file to appear and then executes a command. It provides a simple graphical interface with a progress bar and customizable buttons.

## Features

- Monitors a specified file path
- Executes a command when the file appears
- Shows a progress bar during waiting
- Supports an extra customizable button for additional actions
- Automatically triggers the unlock command on startup (can be disabled)
- Optional icon support (both from file and system icons)
- Keyboard shortcuts (Esc to quit)

## Usage

```bash
wait-for-file -p <presence_file> -c <command> [-e <extra_command>] [-i <icon>] [--no-auto-unlock]
```

### Arguments

- `-p, --presence-file`: The file path to monitor
- `-c, --command`: Command to execute when the file appears
- `-e, --extra-command`: Optional extra command button (format: "Label:command", default: "Unlock:open-vault 120s")
- `-i, --icon`: Optional icon path or icon name
- `--no-auto-unlock`: Disable automatic triggering of the unlock command on startup

### Example

```bash
wait-for-file -p /tmp/trigger -c "echo 'File found!'" -e "Custom:echo 'Custom action'" -i "system-lock-screen" --no-auto-unlock
```

## Extra Command Format

The extra command can be specified in two formats:

1. `Label:command` - Specifies both the button label and the command
2. `command` - Uses "Unlock" as the default label

## Building

Ensure you have Rust and GTK4 development libraries installed, then:

```bash
cargo build --release
```

## Dependencies

- GTK4
- Rust 1.56 or later

## License

MIT
