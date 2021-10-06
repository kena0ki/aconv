# utf8ify  

Converts texts from an auto detected encoding to utf-8 or a specified encoding.  
Since this library depends on `encoding_rs`, available encodings are the ones `encoding_rs` supports.  
If binary files are detected, no conversion of those files takes place, meaning the input data are redirected to output as it is.  
If invalid byte sequences are found, they are replaced with REPLACEMENT CHARACTER(U+FFFD).  


## Options  
```
-f ENCODING, --from-code=ENCODING  
    The candidate encoding of the input.  
    This can be specified multiple times.  
    In addition to supported encodings, special keyword `binary`, which is for binary file, can be specified.  
    If this option is not given, all of the supported encodings and `binary` become candidate,  
    and thus the performance can be pretty bad.  
    Even if utf-8 is not specified, utf-8 becomes always a candidate.  
-t ENCODING, --to-code=ENCODING  
    The encoding of the output.  
    Default is utf-8 as the name implies.  
-o DIRECTORY  
    Output the result to DIRECTORY.  
    If directories exist in inputs, the directory hierarchy is preserved as it is.  
-l  
    Shows supported encodings.  
-e PERCENTAGE, --error-threshold PERCENTAGE  
    Above this threshold (0-100) of invalid byte sequence occurrence,  
    the decoder determines the candidate encoding is not appropriate.  
    If all of the candidate encodings exceeds this threshold and `binary` is a candidate,  
    a file being converted is identified as `binary`.  
    If `binary` is not a candidate, conversion is skipped and error is emitted.  
    The default value is 10.  
```

## How auto-detection works.
Since texts are byte sequences, technically we can not 100% accurately detect the right encoding.  
So we need to guess the right encoding somehow.  
The below is the flow we roughly follow.  

1. Narrow down candidate encodings using the `-f` option.  
2. Do BOM sniffing.  
   If a BOM found, go to 5.  
3. Guess encoding using `chardetng`.  
   If the returned encoding is not in candidates, skip 3, 4, and go to 5.  
4. Choose one of encoding from candidates in turn.  
   If all candidates are already chosen, go to 6.  
5. Decode texts using `encoding_rs`.  
   If REPLACEMENT CHARCTER exceeds `--error-threshold`, quit decoding and return to 4.  
   Otherwise, output decoded texts and exit.  
6. If `binary` is specified by `-f` option, identify the file as a binary file.  
   Otherwise, emit an error message, output texts with no encoding conversion, and exit.  

## LICENSE

