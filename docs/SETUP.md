### Basic Setup Instructions for Contributing to OpenCal

## Installing Rust and Setting up a Development Environment
- Install rust using the [rustup](https://rustup.rs) toolchain manager
  - If you don't already have rust installed follow the prompt and just accept the default options. If you already have rust installed be sure to update your rust version and install the nightly toolchain

  - `$ rustup update`
  - `$ rustup toolchain install nightly --allow-downgrade`

  - Be sure to install clippy (a rust linter) which you can use with the following commands
  - `$ rustup component add clippy`
  - `$ cargo clippy`
  
  - Install a rust code formatter and run it with cargo
  - `$ rustup component add rustfmt`
  - `$ cargo fmt`

There are many more tools out there to help but these are a must-have.

#### Development Environment 

> Note: It is highly recommended you use some sort of UNIX system. It is not required but will almost certainly make things easier for you in the long run.  

VS Code users can install the [rust-analyzer extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

IntelliJ has a rust [IDE](https://www.jetbrains.com/rust/)

The [rust-analyzer project](https://rust-analyzer.github.io/) has installation instructions for many more text-editors
