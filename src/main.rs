use clap::Parser;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::process::Command;
use std::fs;
use std::fs::File;
use std::{thread, time};
use std::cmp;
use md5;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct ArgsOne {
    #[clap(short, long, value_parser)]
    name: String,
    #[clap(short, long, value_parser, default_value_t = 1)]
    count: u8,
}

fn args_test() {
    let args = ArgsOne::parse();

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

fn get_files_in_dir(dir_path: &String) -> std::io::Result<Vec<String>> {
    let paths = fs::read_dir(dir_path)?;
    let mut raw_file_paths = vec![String::new(); 10];
    let mut file_count = 0;
    for _path in paths {
        let path = _path.unwrap().path();
        let path_metadata = fs::metadata(&path)?;
        if path_metadata.is_file() {
            raw_file_paths[file_count] = match path.into_os_string().to_str() {
                Some(v) => String::from(v),
                None => panic!("dir_path not defined"),
            };
            file_count += 1;
        }
    }
    let mut file_paths = vec![String::new(); file_count];
    for i in 0..file_count {
        file_paths[i] = raw_file_paths[i].clone();
    }
    Ok(file_paths)
}

fn get_corresponding_files(src_path: &String, dest_path: &String) -> std::io::Result<Vec<(String, String)>> {
    let src_file_paths = get_files_in_dir(&src_path)?;
    let dest_file_paths = get_files_in_dir(&dest_path)?;
    let mut corr_files = vec![(String::new(), String::new()); cmp::min(src_file_paths.len(), dest_file_paths.len())];
    let mut corr_files_count = 0;
    for src_file_path in &src_file_paths {
        if !src_file_path.ends_with(".glsl") {
            continue;
        }
        for dest_file_path in &dest_file_paths {
            let src_file_name = match src_file_path.split("/").last() {
                Some(v) => v,
                None => panic!("couldn't determine file name from path {}", src_file_path),
            };
            let dest_file_name = match dest_file_path.split("/").last() {
                Some(v) => v,
                None => panic!("couldn't determine file name from path {}", dest_file_path),
            };
            if dest_file_name == format!("{}.js", src_file_name) {
                corr_files[corr_files_count] = (src_file_path.clone(), dest_file_path.clone());
                corr_files_count += 1;
            }
        }
    }
    let mut file_paths = vec![(String::new(), String::new()); corr_files_count];
    for i in 0..corr_files_count {
        file_paths[i] = corr_files[i].clone();
    }
    Ok(file_paths)
}

fn validate_paths(src_path: &String, dest_path: &String) -> std::io::Result<bool> {
    let src_metadata = fs::metadata(src_path)?;
    let dest_metadata = fs::metadata(dest_path)?;
    if (src_metadata.is_dir() && dest_metadata.is_dir()) {
        return Ok(true);
    } else if src_metadata.is_dir() || dest_metadata.is_dir() {
        return Ok(false);
    } else {
        if (!src_path.ends_with(".glsl") && dest_path.ends_with(".js")) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn list_files_test() -> std::io::Result<()> {
    let args = Args::parse();
    let src_path_os = args.src_path.into_os_string();
    let src_path = match src_path_os.to_str() {
        Some(v) => String::from(v),
        None => panic!("src_path not defined"),
    };
    let paths = get_files_in_dir(&src_path)?;
    for file_path in paths.into_iter() {
        println!("{}", file_path);
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let src_path_os_str = args.src_path.into_os_string();
    let src_path = match src_path_os_str.to_str() {
        Some(v) => String::from(v),
        None => panic!("src_path not defined"),
    };
    let dest_path_os_str = args.dest_path.into_os_string();
    let dest_path = match dest_path_os_str.to_str() {
        Some(v) => String::from(v),
        None => panic!("dest_path not defined"),
    };
    
    if !validate_paths(&src_path, &dest_path)? {
        panic!("invalid paths:\n    {}\n    {}", src_path, dest_path);
    }

    let src_metadata = fs::metadata(&src_path)?;

    println!("Webglsl Daemon started: Building from {} to {}", src_path, dest_path);
    thread::sleep(time::Duration::from_secs(5));

    if src_metadata.is_dir() {
        let file_path_pairs = get_corresponding_files(&src_path, &dest_path)?;
        println!("Tracking files:");
        for (src_path, dest_path) in &file_path_pairs {
            println!("    {} to {}", &src_path, &dest_path);
        }
        let mut md5_hashes = vec![String::new(); file_path_pairs.len()];
        loop {
            for i in 0..file_path_pairs.len() {
                let (src_path, dest_path) = file_path_pairs[i].clone();
                let mut src_file = File::open(&src_path)?;
                let mut src_contents = String::new();
                src_file.read_to_string(&mut src_contents)?;
                
                let contents_md5 = format!("{:x}", md5::compute(&src_contents));
                if contents_md5 != md5_hashes[i] {
                    md5_hashes[i] = contents_md5.clone();
                    let mut dest_file = File::options().write(true).open(&dest_path)?;
                    let out_str = format!("export default`{}`;", src_contents);
                    dest_file.set_len(0)?;
                    dest_file.write_all(out_str.as_bytes())?;
                }
            }
            thread::sleep(time::Duration::from_secs(1));
        }
    } else {
        let mut last_md5 = String::new();
        loop {
            let mut src_file = File::open(&src_path)?;
            let mut src_contents = String::new();
            src_file.read_to_string(&mut src_contents)?;
            // println!("{}", src_contents);

            let contents_md5 = format!("{:x}", md5::compute(&src_contents));
            if contents_md5 != last_md5 {
                last_md5 = contents_md5.clone();
                let mut dest_file = File::options().write(true).open(&dest_path)?;
                let out_str = format!("export default `{}`;", src_contents);
                dest_file.set_len(0)?;
                dest_file.write_all(out_str.as_bytes())?;
            }
            thread::sleep(time::Duration::from_secs(1));
        }
    }
}

fn file_type_test() -> std::io::Result<()> {
    let args = Args::parse();
    let src_path_os_str = args.src_path.into_os_string();
    let src_path = match src_path_os_str.to_str() {
        Some(v) => v,
        None => panic!("src_path not defined"),
    };
    let metadata = fs::metadata(src_path)?;
    let file_type = metadata.file_type();
    println!("is_dir: {}", file_type.is_dir());
    Ok(())
}
