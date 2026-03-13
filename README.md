# Tamagotogether

Tamagotogether is a collaborative Tamagotchi platform built with Rust. It allows multiple players to care for a single pet together through a web interface.

## Features

- **Daily Mood Reset**: The pet's mood is randomized daily between level 1 and 5 (out of 10).
- **Collaborative Feeding**: Each player can feed the pet once per day (tracked by hashed IP).
- **Happiness Evolution**: Each feeding increases the happiness level by 1, up to a maximum of 10.
- **Web Interface**: A responsive UI that provides real-time status and feedback.
- **Privacy Hashing**: IP addresses are hashed using SHA-256 for privacy before being stored in the database.

## Technical stack

- **Backend**: Rust using Axum for the web server and Rusqlite for database access.
- **Frontend**: Vanilla HTML, CSS, and JavaScript.
- **Database**: SQLite.
- **Asset serving**: Static files are embedded in the Rust binary using rust-embed.

## Getting started

### Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd tamagotogether
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

### Running locally

```bash
cargo run
```
The application will listen on port 8080 by default.

## Project structure

- `src/`: Rust backend source code.
    - `main.rs`: Server initialization and routing.
    - `handlers.rs`: API endpoint logic.
    - `models.rs`: Data structures and shared state.
    - `db.rs`: Database operations.
- `static/`: Frontend assets (embedded in the final binary).

## Roadmap

- Interactive canvas for pet animations.
- Additional collaborative actions (playing, cleaning).
- Global activity history.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.