# Abacus Simulator

[![Live Demo](https://img.shields.io/badge/Demo-Live-brightgreen)](https://swultcode.github.io/Abacus-Simulator/)

A 3D interactive abacus simulator built with Bevy Engine and Rust, compiled to WebAssembly for browser-based use.

## Usage

You can access the live demo at: [https://swultcode.github.io/Abacus-Simulator/](https://swultcode.github.io/Abacus-Simulator/)

### Controls

- **Click** beads to move them
- Use the **Abacus Settings panel** to customize the abacus layout

## Educational Applications

This is a personal project, I'm not an abacus expert so historical accuracy is not guaranteed.

This abacus simulator has been designed for educational purposes, currently through configuring you can explore:
- The Suanpan (or Chinese abacus) in 2/5 configuration
- The Soroban (or Japanese abacus) in 1/4 configuration
- A binary counter
- Your own custom abacus's

## Building

If you want to build the project yourself, run the following:

```
cargo build --release --target wasm32-unknown-unknown
```

```
wasm-bindgen --out-dir ./webbuild/out/ --target web ./target/wasm32-unknown-unknown/release/Abacus-Simulator.wasm 
```

```
npx serve webbuild
```

Now you should be able to see the abacus in your local network!

## License

[MIT License](LICENSE)

## Acknowledgments

- [This youtube tutorial by Biped Potato](https://www.youtube.com/watch?v=VjXiREbPtJs) for building to web