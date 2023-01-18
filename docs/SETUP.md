### Basic Setup Instructions for Contributing to OpenCal

#### Installing Rust and Setting up a Development Environment
- Install rust using the [rustup](https://rustup.rs) toolchain manager
  - If you don't already have rust installed follow the prompt and just accept the default options
  - If you already have rust installed be sure to update your rust version and install the nightly toolchain

  - `$ rustup update`
  - `$ rustup toolchain install nightly --allow-downgrade`

  - Be sure to install clippy (a rust linter)
  - `$ rustup component add clippy`
  - You can run clippy on your project using 
  - `$ cargo clippy`
  
  - Install a rust code formatter
  - `$ rustup component add rustfmt`
  - and format your project with
  - `$ cargo fmt`

There are many more tools out there to help but these are a must-have.


- For our CI pipeline we use Github Actions
