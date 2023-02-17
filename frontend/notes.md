# Notes on Sycamore and developing frontend in Rust
Following tutorials at https://sycamore-rs.netlify.app/#

### Basics
Include `use sycamore::prelude::*;` in a rust file

Basic hello world script:
```
fn main() {
    sycamore::render(|cx| view! { cx,
        p { "Hello, World!" }
    });
}
```
Breakdown:
- `main()` starts the application
- `sycamore::render(...)` built in render function to render stuff to the DOM
- `view!` macro for creating user interfaces
- `cx` reactive scope
- `p { "Hello, World!" }` is equivalent to `<p>Hello World!</p>`

**Sycamore needs an HTML file to inject the code into**

Run `trunk serve` from the command line to run, access at localhost:8080
