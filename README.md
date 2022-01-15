# WFC

The wfc-rs crate is a wrapper for the [krychu/wfc](https://github.com/krychu/wfc) implementation
of Wave Function Collapse.

The wfc library is manually wrapped with extern functions, and a
small, more ideomatic Rust wrapper is provided.

Note that this is an early version. For example, dropping a Wfc struct 
will not clean up memory.

