use clap::Parser;
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    name: String,

    #[clap(short, long, value_parser, default_value_t = 1)]
    count: u8,
}

fn main_one() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name);
    }
}

fn main() {
    let output = Command::new("touch")
        .arg("test.txt")
        .output()
        .expect("touch failed");
    let output1 = Command::new("echo")
        .arg("nice to see you around these parts")
        .arg(">>")
        .arg("test.txt")
        .output()
        .expect("echo failed");
    let stdout = match std::str::from_utf8(&output.stdout) {
        Ok(v) => v,
        Err(err) => panic!("Invalid utf-8 sequence {}", err),
    };
    let stderr = match std::str::from_utf8(&output1.stderr) {
        Ok(v) => v,
        Err(err) => panic!("Invalid utf-8 sequence {}", err),
    };
    println!("{}", stdout);
    println!("{}", stderr);
}