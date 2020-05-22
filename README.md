# Safe bindings to `WSAPoll`

You want to `#![forbid(unsafe_code)]` in your crate? But you also need access to
`WSAPoll()`? Then this crate is for you! It exports a safe `wsa_poll()` function
that you can use.
