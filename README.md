# chip8
A Chip8 emulator (interpreter) written in Rust

##![chip8-demo](https://user-images.githubusercontent.com/51889103/193450813-22d66fa8-b8ab-42c0-9325-b5ae144b9649.gif

Usage 
```
chip8.exe [OPTIONS] <path>

Arguments:
   <path>  Path of rom file

Options:
  -l, --legacy                   Run with old instructions on
  -f, --frequency [<FREQUENCY>]  Run with specified frequency [default: 700]
  -h, --help                     Print help information
  -V, --version                  Print version information
```


## Build
```
cargo build --release
```
If you get errors about SFML headers not being found, or linker errors, that probably means SFML is not installed in a global location. In that case, you can set two environment variables to help rust-sfml find the required files:

```
SFML_INCLUDE_DIR. Set this to the include folder of your SFML location.
SFML_LIBS_DIR. Set this to the lib folder of your SFML location.
```
