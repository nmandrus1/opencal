

use sycamore::prelude::*;

fn main() {
    sycamore::render(|cx| view! { cx,
        p { "Hello, World!" }
        button { "My Button!" }
    });
}