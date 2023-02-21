# RustNESs
NES emulator written in rust.

<!-- ![Super Mario Bros.]( "Super Mario Bros.") -->
<img src="https://i.ibb.co/XCmK8Dw/Bildschirmfoto-2023-02-15-um-21-08-00.png" width=50% height=50%>

## Instructions

You must have `rustup` installed. If not, you can [follow this tutorial](https://doc.rust-lang.org/book/ch01-01-installation.html#installation) to get it.

- Run the following commands to download the code:
```
git clone https://github.com/Bensas/RustNESs
cd RustNESs
```

- And then run
```
cargo run --release <path-to-ROM-file>
```
to open the emulator.

### Key bindings
| Button  | Key mapping |
| ------------- | ------------- |
| A   | N  |
| B  | M  |
| Start  | J  |
| Select  | H  |
| Up  | W  |
| Left  | A  |
| Down  | S  |
| Right  | D  |



## Roadmap of upcoming features:
- APU implementation to have sound.
- Support for more mappers (currently only supports Mapper000).
- UI toggle to display system information vs just the screen.
- UI widget to upload ROM file instead of passing it as CL argument.

## Resources utilized:
- https://www.youtube.com/@javidx9
- https://www.nesdev.org/wiki/Nesdev_Wiki
- http://datasheets.chipdb.org/Rockwell/6502.pdf
- https://www.masswerk.at/6502/
