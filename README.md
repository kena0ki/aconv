# aconv  

Converts texts from the auto-detected encoding to UTF-8 or a specified encoding.  
Since this library depends on [`encoding_rs`](https://github.com/hsivonen/encoding_rs), available encodings are the ones defined in [the Encoding Standard](https://encoding.spec.whatwg.org).  

Note: UTF-16 files are needed to have a BOM to be automatically detected as the encoding. This is because [`chardetng`](https://github.com/hsivonen/chardetng) does not support UTF-16 and this library added only BOM sniffing to detect UTF-16.  

## Installation
```
cargo install aconv
```


## Usage
```
aconv 0.1.0
Converts texts from the auto detected encoding to UTF-8 or a specified encoding.
If malformed byte sequences are found, they are replaced with the REPLACEMENT CHARACTER(U+FFFD).
If the auto-detection is considered it failed, the input texts are output as-is,
meaning no conversion takes place, with an error message emitted.

USAGE:
    aconv [FLAGS] [OPTIONS] [FILE]...

FLAGS:
    -h, --help       Prints help information
    -l, --list       Prints supported encodings
    -q, --quiet      Suppresses error messages when auto-detection failed
    -s, --show       Only shows auto-detected encodings without decoded texts
    -v, --version    Prints version info and exit

OPTIONS:
    -o, --output <DIRECTORY>                 Output directory. If input arguments contain directories, the directory
                                             hierarchies are preserved under DIRECTORY
    -t, --to-code <ENCODING>                 The encoding of the output [default: UTF-8]
    -c, --chars-to-guess <NUMBER>            Number of non-textual ascii characters to guess the encoding. Around 100
                                             characters are enough for most cases, but if the guess is not accurate,
                                             increasing the value might help [default: 100]
    -T, --non-text-threshold <PERCENTAGE>    Threshold (0-100) of non-text character occurrence. Above this threshold in
                                             decoded texts, the auto-detection is treated as it failed. In that case the
                                             input texts are output as-is with an error message emitted [default: 0]

ARGS:
    <FILE>...    Files (or directories) to process
```


## How auto-detection works  
See [transcoding_rs](transcoding_rs/README.md#how-auto-detection-works).


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

