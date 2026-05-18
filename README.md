# Aeropeks

A premium macOS-style top menu bar for Windows, built with Tauri, React, and Rust.

Aeropeks brings the elegance and functionality of the macOS menu bar to Windows. It provides a centralized hub for system status, media control, terminal access, and more, all while maintaining a sleek, unobtrusive design.

## Key Features

*   **System Status at a Glance:** Monitor battery, Bluetooth, and volume directly from the bar.
*   **Media Control:** Unified media player integration with support for Plex. Control playback and see what's playing without leaving your current app.
*   **Weather Integration:** Real-time weather updates powered by met.no/yr.no.
*   **Privacy First:** Quick-toggle "Privacy Mode" to block camera and microphone access.
*   **Integrated Terminal:** A drop-down terminal panel for quick command execution and SSH shortcuts.
*   **OBS Integration:** Visual indicators for OBS streaming and recording status.
*   **Power Management:** Quick access to Lock, Sleep, Restart, and Shutdown commands.
*   **Highly Customizable:** Accent colors, terminal shortcuts, and weather locations can be configured in settings.

## Getting Started

### Prerequisites

To build Aeropeks from source, you will need:
*   [Node.js](https://nodejs.org/) (v18+)
*   [Rust](https://www.rust-lang.org/)
*   [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) (Windows)

### Installation

1.  Clone the repository:
    ```bash
    git clone https://github.com/Antigravity/Aeropeks.git
    cd Aeropeks
    ```

2.  Install frontend dependencies:
    ```bash
    npm install
    ```

3.  Run in development mode:
    ```bash
    npm run tauri dev
    ```

4.  Build for production:
    ```bash
    npm run tauri build
    ```

## Configuration

Aeropeks stores its settings locally in `~/.aeropeks/settings.json`. You can also configure most settings through the built-in Settings window.

## License

This project is [Private/Specify License].
