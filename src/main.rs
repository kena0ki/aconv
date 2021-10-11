use structopt::StructOpt;

fn main() {
    let opt = utf8ify::Opt::from_args();
    println!("{:?}", opt);
    utf8ify::cli(opt);
}

