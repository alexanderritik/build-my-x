use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::Read;
use std::path::PathBuf;
use std::vec;
use bytes::buf;
use clap::builder::Str;
use clap::{Parser, Subcommand};
use flate2::read::{ZlibDecoder, ZlibEncoder};
use anyhow::Context;
use sha1::{Sha1, Digest};
use flate2::Compression;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

/// Doc comment
#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile{
        object_hash: String,

        #[clap(short= 'p')]
        pretty_print: bool,
    },
    HashObject{
        file_name: PathBuf,

        #[clap(short= 'w')]
        pretty_print: bool,
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    // println!("{:?}", args);
    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        },
        Command::CatFile { pretty_print, object_hash} => {
            // println!("cat-file {} {:?}", pretty_print, object_hash);
            let f = fs::File::open(format!(
                    ".git/objects/{}/{}",
                    &object_hash[..2], 
                    &object_hash[2..]
                )).context("context")?;

            let buffer = ZlibDecoder::new(f);
            let mut reader = std::io::BufReader::new(buffer);
            
            let mut contents = Vec::new();
            reader.read_to_end(&mut contents)?;

            // Find the position of the first null byte ('\0')
            if let Some(pos) = contents.iter().position(|&b| b == 0) {
                let file_contents = &contents[pos + 1..];
                print!("{}", String::from_utf8_lossy(file_contents).trim_end());
            } else {
                println!("Invalid Git blob format");
            }
        }
        Command::HashObject { pretty_print, file_name} => {

            let stat = std::fs::metadata(&file_name).context("context")?;
            let f = fs::File::open(&file_name).context("context")?;
            

            let mut hasher = Sha1::new();
            hasher.update(format!("blob {}\0{}", stat.len(), file_name.display()).as_bytes());

            let mut e = ZlibEncoder::new(vec![], Compression::default());
            // e.write(format!("blob {}\0{}", stat.len(), file_name.display()).as_bytes());

            //We need to compress it and then add it to the objects directory
            let header = "blob".to_string();


            let buffer = std::io::BufReader::new(f);
            let mut reader = std::io::BufReader::new(buffer);        
            let mut contents = Vec::new();
            reader.read_to_end(&mut contents)?;

            let hashCode = ZlibEncoder
                ::new(f, flate2::Compression::default()).read_to_end(&mut contents).context("Faile");
        }
    }
    Ok(())

}
