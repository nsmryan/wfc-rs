# WFC

The wfc-rs crate is a wrapper for the [krychu/wfc](https://github.com/krychu/wfc) implementation
of Wave Function Collapse.

The wfc library is manually wrapped with extern functions, and a
small, more ideomatic Rust wrapper is provided.

Note that this is an early, but working version.


## Usage

Once the crate has been added to Cargo.toml as:
```
wfc_rs = "0.3"
```

create a WfcImage with from_vec or from_file:
```rust
let image = WfcImage::from_image("data/cave.png")?;
```
This creates an optional NonNull, which contains a pointer to the underlying
WfcImage structure. This structure is a repr(C) struct that matches the 'wfc.h'
structure 'wfc_image'.

and then a Wfc structure from this image, as well as the configuration settings:
```rust
    let mut wfc = Wfc::overlapping(32, 32, image, 3, 3, true, true, true, true)?;
```

With this Wfc, you can run an interation with 'run'
```rust
wfc.run();
```
and either export an input:
```rust
wfc.export("output.png");
```
or get the raw pixel data as a Vec<u8>:
```rust
let bytes = wfc.vec();
```

