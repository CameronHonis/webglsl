use clap::Parser;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::process::Command;
use std::fs::File;
use std::{thread, time};
use md5;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args_One {
    #[clap(short, long, value_parser)]
    name: String,

    #[clap(short, long, value_parser, default_value_t = 1)]
    count: u8,
}

fn args_test() {
    let args = Args_One::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name);
    }
}

fn commands_test() {
    let output = Command::new("ls")
        .output()
        .expect("pwd failed");
    let stdout = match std::str::from_utf8(&output.stdout) {
        Ok(v) => v,
        Err(err) => panic!("Invalid utf-8 sequence {}", err),
    };
    println!("{}", stdout);
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(parse(from_os_str))]
    src_path: std::path::PathBuf,
    #[clap(parse(from_os_str))]
    dest_path: std::path::PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let src_path_os_str = args.src_path.into_os_string();
    let src_path = match src_path_os_str.to_str() {
        Some(v) => {
            if !v.ends_with(".glsl") {
                panic!("src isn't a .glsl file");
            }
            v
        },
        None => panic!("src_path not defined"),
    };
    let dest_path_os_str = args.dest_path.into_os_string();
    let dest_path = match dest_path_os_str.to_str() {
        Some(v) => {
            if !v.ends_with(".js") {
                panic!("dest isn't a .js file");
            }
            v
        },
        None => panic!("dest_path not defined"),
    };
    File::open(src_path)?;
    File::open(dest_path)?;

    println!("Webglsl Daemon started: Building from {} to {}", src_path, dest_path);
    thread::sleep(time::Duration::from_secs(5));

    let mut last_md5 = String::new();
    loop {
        let mut src_file = File::open(src_path)?;
        let mut src_contents = String::new();
        src_file.read_to_string(&mut src_contents)?;
        // println!("{}", src_contents);

        let contents_md5 = format!("{:x}", md5::compute(&src_contents));
        if contents_md5 != last_md5 {
            last_md5 = contents_md5.clone();
            let mut dest_file = File::options().write(true).open(dest_path)?;
            let out_str = format!("export default `{}`;", src_contents);
            dest_file.set_len(0)?;
            dest_file.write_all(out_str.as_bytes())?;
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}
