use structopt::StructOpt;
use aconv::option;
use aconv::cli;

fn main() -> () {
    let opt: option::Opt = StructOpt::from_args();
    println!("{:?}", opt);
    match cli::dispatch(&opt) {
        Err(err) => {
            if ! err.is_guess() {
                eprintln!("{}", err);
            }
            std::process::exit(err.error_code());
        },
        Ok(_) => {
            std::process::exit(0);
        },
    };
}

