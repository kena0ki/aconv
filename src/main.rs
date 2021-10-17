use structopt::StructOpt;
use utf8ify::option;

fn main() -> () {
    // let opt = utf8ify::option::Opt::default();
    let opt: option::Opt = StructOpt::from_args();
    println!("{:?}", opt);
    match utf8ify::cli(&opt) {
        Err(err) => eprintln!("{}", err),
        Ok(_) => {},
    };
}

