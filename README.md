# aconv  

Converts texts from the auto-detected encoding to UTF-8 or a specified encoding.  
This is similar to the `iconv` command but differences are following.  
  - Detects encoding if the source encoding is not specified.
  - Replaces malformed byte sequences with the REPLACEMENT CHARACTER or the corresponding numeric character reference, which depends on the destination charset (i.e. Unicode or not).
  - Can recursively convert files in directories and output converted files to the specified directory preserving the directory hierarchy.

Since this library depends on [`encoding_rs`](https://github.com/hsivonen/encoding_rs), available encodings are the ones defined in [the Encoding Standard](https://encoding.spec.whatwg.org).  

Note: UTF-16 files are needed to have a BOM to be detected as the encoding.  
      This is because [`chardetng`](https://github.com/hsivonen/chardetng), on which this library depends, does not support UTF-16 and this library only added BOM sniffing to detect UTF-16.  


## Installation
```
cargo install aconv
```


## Usage
```
Converts texts from the auto-detected encoding to UTF-8 or a specified encoding.
If byte sequences that is malformed as Unicode are found,
they are replaced with the REPLACEMENT CHARACTER(U+FFFD).
If the destination encoding is not Unicode and unmappable characters are found, they are
replaced with the corresponding numeric character references.
If the encoding detection is considered it failed, the input texts are output as-is,
meaning no conversion takes place, and an error message is emitted.

USAGE:
    aconv [FLAGS] [OPTIONS] [FILE]...

FLAGS:
    -h, --help       Prints help information
    -l, --list       Prints supported encodings
    -q, --quiet      Suppresses error messages when encoding detection failed
    -s, --show       Only shows auto-detected encodings without decoded texts
    -V, --version    Prints version information

OPTIONS:
    -o, --output <DIRECTORY>                 Output directory. If input arguments contain directories, the directory
                                             hierarchies are preserved under DIRECTORY
    -t, --to-code <ENCODING>                 The encoding of the output [default: UTF-8]
    -A, --non_ascii_to_guess <NUMBER>        The number of non-ASCII characters to guess the encoding. Around 100
                                             characters are enough for most cases, but if the guess is not accurate,
                                             increasing the value might help [default: 100]
    -T, --non-text-threshold <PERCENTAGE>    The threshold (0-100) of non-text character occurrence. Above this
                                             threshold in decoded UTF-8 texts, the encoding detection is treated as it
                                             failed. In that case the input texts are output as-is with an error message
                                             emitted [default: 0]

ARGS:
    <FILE>...    Files (or directories) to process
```


## How encoding detection works  
See [transcoding_rs](transcoding_rs/README.md#how-encoding-detection-works).


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

