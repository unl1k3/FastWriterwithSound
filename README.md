# Mechanical Keyboard Sound Simulator

This application allows users to simulate the sound of a mechanical keyboard by associating custom audio files to key presses.
Users can assign any audio file to a specific key, enabling a personalized typing experience with realistic mechanical keyboard sounds.
Ideal for those who want to add a tactile audio dimension to their typing, without the need for a physical mechanical keyboard.
This application is a simplified version of [Mechvibes DX](https://github.com/hainguyents13/mechvibes-dx) created by Hải Nguyễn.

## Installation and Compilation

1. **Install Rust**
   Install Rust by following the official instructions at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install). This will set up the Rust toolchain and the `cargo` package manager.

2. **Clone the Repository**
   Download the project source code using Git:
   ```bash
   git clone git@github.com:unl1k3/KeyTapSound.git
   cd KeyTapSound

4. ** Compile & Run the Application**
You can run the program directly with Cargo:
    ```bash
    cargo run --release

## Required File Layout

```text
KeyTapSound/
├── config.toml
├── keytap
└── soundtrack/
    └── eg-crystal-purple/
        ├── config.json
        └── sound.ogg
# License

This project is licensed under the MIT License.
