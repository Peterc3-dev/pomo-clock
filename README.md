# pomo-clock

Pomodoro timer TUI with large ASCII digit display and session tracking.

## Features

- Big-digit countdown rendered in the terminal — visible across the room
- Configurable work, short break, and long break durations
- Automatic phase cycling: work -> short break -> ... -> long break
- Session statistics view (Tab) tracking completed pomodoros
- Adjust remaining time on the fly with `+` / `-` (1 minute increments)
- Optional auto-start between phases (`--auto-start`)
- Optional shell command on phase completion (`--notify-cmd`)
- Persistent session stats saved to `~/.config/pomo-clock/`

## Install

```
cargo build --release
# binary at target/release/pomo-clock
```

## Usage

```
# default: 25min work / 5min short / 15min long / 4 sessions
pomo-clock

# custom durations
pomo-clock --work 50 --short-break 10 --long-break 30

# auto-start + desktop notification
pomo-clock --auto-start --notify-cmd "notify-send 'Phase complete'"
```

## Keybindings

| Key | Action |
|-----|--------|
| `Space` | Pause / resume |
| `s` | Skip current phase |
| `r` | Reset current phase |
| `+` / `-` | Add / subtract 1 minute |
| `Tab` | Toggle stats view |
| `q` | Quit |

---
Built with Rust + ratatui
