# macro_railroad Annotations
Annotation macro to generate railroads for macro_rules! and embed them in the documentation

## Usage
```rs
/// This is some documentation for the macro below
#[macro_railroad_annotation::generate_railroad]
/// # Examples
/// ````ignore
/// foo!(5 + 5);
/// ````
macro_rules! foo {
  ($tok:expr) => {};
}
```

Be aware that this only positions the graphic correctly on nightly compilers (which includes docs.rs).
If on stable the graphic will just be positioned at the top of the doc comments.
The alt text will be set to the following banner:
```
=========================================================
_Here would be a railroad diagram of the macro [`macro_name`]_
=========================================================
```

Alternatively, you can position the image manually and specify your own alt text, which works on all versions:
```rs
/// This is some documentation for the macro below
/// ![alt text][ref_string]
/// # Examples
/// ````ignore
/// foo!(5 + 5);
/// ````
#[macro_railroad_annotation::generate_railroad("ref_string")]
macro_rules! foo {
  ($tok:expr) => {};
}
```

# Credits
This macro uses the excellent [`macro_railroad`](https://crates.io/crates/macro_railroad) crate to generate the diagrams, and is only a slim wrapper around it.
