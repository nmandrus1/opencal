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


### Views
Div examples:
```
view! { cx,
    // A simple div
    div {}
    // A div with a class
    div(class="foo")
    // An empty paragraph
    p {}
    // Custom elements!
    my-custom-element {}
}
```

Nesting example:
```
view! { cx,
    div {
        p {
            span { "Hello " }
            strong { "World!" }
        }
    }
}
```

You can insert code into views if it has a `std::fmt::display` property:
```
let my_number = 123;

view! { cx,
    p {
        "This is my number: " (my_number)
    }
}
```

Can even nest views inside views:
```
let inner_view = view! { cx,
    "Inside"
};

let outer_view = view! { cx,
    "Outside"
    div {
        (inner_view)
    }
};
```


