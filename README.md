# Red Alert RS

A Red Alert 2 inspired RTS game built with Rust + [Bevy](https://bevyengine.org).

## Tech Stack

- **Language:** Rust (edition 2021)
- **Engine:** Bevy 0.14 (ECS architecture)
- **CI/CD:** GitHub Actions (fmt + clippy + build + test)

## Current Features

- [x] 2D tile-based map grid (50x50)
- [x] Camera setup
- [x] Unit spawning
- [x] Click-to-move (simplified)
- [ ] Pathfinding (A*)
- [ ] Resource gathering (ore mining)
- [ ] Building construction
- [ ] Combat system
- [ ] Fog of war
- [ ] AI opponent
- [ ] Networking (multiplayer)

## Development

```bash
cargo run
```

## CI

GitHub Actions runs on every push/PR:
- `cargo fmt --check`
- `cargo clippy -D warnings`
- `cargo check`
- `cargo test`
