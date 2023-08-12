# HandleMyShell!
TUI reverse shell handler in Rust

> Note: This is a work in progress


## Installation

```shell
cargo install --git https://github.com/KMikeeU/handlemyshell
```



## Usage

Start handlemyshell:

```shell
hms
```



## Keybindings

### Global Keybindings
| Keybinding | Description                     |
| ---------- | ------------------------------- |
| `CTRL+a`   | Switch between Local and Remote |

### Local-Tab Keybindings

| Keybinding | Description                                  |
| ---------- | -------------------------------------------- |
| `q`        | Exit handlemyshell                           |
| `<TAB>`    | Switch between "Listeners" and "Session" tab |

### Listeners-Tab Keybindings

| Keybinding            | Description                   |
| --------------------- | ----------------------------- |
| `c`                   | Create a listener             |
| `UP_ARROW/DOWN_ARROW` | Move in the list of listeners |


### Session-Tab Keybindings

| Keybinding            | Description                  |
| --------------------- | ---------------------------- |
| `UP_ARROW/DOWN_ARROW` | Move in the list of sessions |
| `<SPACE>`             | Select a session             |

