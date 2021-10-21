# transcoding_rs  

Detects the source text encoding and transcodes to another.  
Since this library depends on [`encoding_rs`](https://github.com/hsivonen/encoding_rs), available encodings are the ones defined in [the Encoding Standard](https://encoding.spec.whatwg.org).  

Note: UTF-16 files are needed to have a BOM to be automatically detected as the encoding. This is because [`chardetng`](https://github.com/hsivonen/chardetng) does not support UTF-16 and this library added only BOM sniffing to detect UTF-16.  

## Usage

## How auto-detection works.  
Since texts are internally just byte sequences, there is no way to detect the right encoding with 100% accuracy.  
So we need to guess the right encoding somehow.  
The below is the flow we roughly follow.  

1. Do BOM sniffing. This is needed because [`chardetng`](https://github.com/hsivonen/chardetng) does not detect UTF-16.  
   If a BOM is found, skip guessing the encoding.  
2. Guess the encoding using `chardetng`.  
3. Decode texts using `encoding_rs`.  
4. Check the decoded texts if there are non-text characters, which are described below.  
   If non-text characters do not exceed the threshold, output the decoded texts.  
   Otherwise, emit an error message and output the input texts as it is.  

#### Non-text characters  
Characters that are treated as non-text in this library are the same [ones](https://github.com/file/file/blob/ac3fb1f582ea35c274ad776f26e57785c4cf976f/src/encoding.c#L236) in the `file` command, plus REPLACEMENT CHARACTER.  
Namely, U+0000 ~ U+0006, U+000e ~ U+001a, U+001c ~ U+001f, U+007f, and U+FFFD are treated as the non-text characters.  


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

