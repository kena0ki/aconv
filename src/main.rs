use structopt::StructOpt;
use utf8ify::option;

fn main() -> () {
    // let opt = utf8ify::option::Opt::default();
    let opt: option::Opt = StructOpt::from_args();
    println!("{:?}", opt);
    match utf8ify::cli::dispatch(&opt) {
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

