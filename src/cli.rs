fn cli() {
    let mut file;
    let stdout;
    let mut stdout_lock;
    let mut output: &mut dyn Write = match matches.opt_str("o").as_ref().map(|s| &s[..]) {
        None | Some("-") => {
            stdout = std::io::stdout();
            stdout_lock = stdout.lock();
            &mut stdout_lock
        }
        Some(path_string) => {
            match File::create(&Path::new(path_string)) {
                Ok(f) => {
                    file = f;
                    &mut file
                }
                Err(_) => {
                    print!("Cannot open {} for writing; exiting.", path_string);
                    std::process::exit(-3);
                }
            }
        }
    };

    let mut decoder = input_encoding.new_decoder();
    let mut encoder = output_encoding.new_encoder();

    if matches.free.is_empty() {
        convert(&mut decoder,
                &mut encoder,
                &mut std::io::stdin(),
                output,
                true,
                use_utf16);
    } else {
        let mut iter = matches.free.iter().peekable();
        loop {
            match iter.next() {
                None => {
                    break;
                }
                Some(path_string) => {
                    match &path_string[..] {
                        "-" => {
                            convert(&mut decoder,
                                    &mut encoder,
                                    &mut std::io::stdin(),
                                    &mut output,
                                    iter.peek().is_none(),
                                    use_utf16);
                        }
                        _ => {
                            match File::open(&Path::new(&path_string)) {
                                Ok(mut file) => {
                                    convert(&mut decoder,
                                            &mut encoder,
                                            &mut file,
                                            &mut output,
                                            iter.peek().is_none(),
                                            use_utf16);
                                }
                                Err(_) => {
                                    print!("Cannot open {} for reading; exiting.", &path_string);
                                    std::process::exit(-4);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    match output.flush() {
        Ok(_) => {}
        Err(_) => {
            print!("Cannot flush output; exiting.");
            std::process::exit(-3);
        }
    }
}

