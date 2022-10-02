# chip8
A Chip8 emulator (interpreter) written in Rust

## Usage 
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
cabal install --lib sdl2
cabal install --lib sdl2-ttf
cabal install --lib aeson
cabal install --lib bytestring
cabal install --lib utf8-string
cabal install --lib linear
cabal install --lib vector
ghc --make scov
```