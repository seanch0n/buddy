use base64::{decode, encode};
use checksums::{hash_file, Algorithm::SHA1};
use clap::{App, Arg};
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str;

fn main() {
    let app = App::new("buddy")
        .version("0.1")
        .about("send files")
        .author("who knows");

    let server_option = Arg::with_name("server")
        .long("server") // allow --server
        .takes_value(false)
        .help("run as a server receiving files")
        .required(false);
    let app = app.arg(server_option);

    let client_option = Arg::with_name("client")
        .long("client") // allow --client
        .takes_value(false)
        .help("run as a client to send files")
        .required(false);
    let app = app.arg(client_option);

    let matches = app.get_matches();

    let server = matches.is_present("server");

    println!("Hello, {}!", server);

    let file_path = "sending.txt";
    if matches.is_present("client") {
        send_file(file_path);
    } else if matches.is_present("server") {
        println!("recv_file is getting called!");
        recv_file(file_path.to_string());
    }
}

fn send_file(file_path: &str) -> io::Result<()> {
    let path = Path::new(file_path);
    let mut file = File::open(path)?;
    let mut tcp = TcpStream::connect("127.0.0.1:4321")?;
    let mut buf = [0; 4096];
    println!("We are sending the file!");
    let contents = fs::read_to_string(path).unwrap();
    let contents_slice: &str = &contents[..];
    // get the sha1 hash of the file being transferred
    let file_hash = hash_file(path, SHA1);
    let file_hash_slice: &str = &file_hash[..];

    // create a payload that is the 'FileHashData'
    let payload = [file_hash_slice, &contents_slice].concat();
    // then encode the payload
    let encoded_payload = base64::encode(&payload);
    // send it
    tcp.write_all(&payload.as_bytes())?;
    // loop {
    //     let n = file.read(&mut buf)?;
    //     if n == 0 {
    //         // reached end of file
    //         break;
    //     }
    //     // get the sha1 hash of the file being transferred
    //     let file_hash = hash_file(path, SHA1);
    //     let file_hash_slice: &str = &file_hash[..];

    //     // create a payload that is the 'FileHashData'
    //     let payload = [file_hash_slice.as_bytes(), &buf].concat();
    //     // then encode the payload
    //     let encoded_payload = base64::encode(&payload);
    //     // send it
    //     tcp.write_all(&encoded_payload[..n].as_bytes())?;
    // }
    Ok(())
}

fn recv_file(file_path: String) -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4321").unwrap();
    println!("Listening on port 4321");
    for stream in listener.incoming() {
        println!("We have a connection!");
        let stream = stream.unwrap();
        handle_connection(stream, file_path.clone());
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, file_path: String) {
    // let mut buffer = [0; 4096];
    let HASH_LENGTH = 40;
    let mut data = vec![];
    let mut data_len = 0usize;
    let mut buf = [0; 4096];

    loop {
        let n = stream.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        data.extend_from_slice(&buf[..n]);
    }

    let path = Path::new("recvd.txt");
    fs::write(path, &data[HASH_LENGTH..]).expect("Unable to write file");
    // the buffer coming in is base64 encoded
    // let decoded_buffer = base64::decode(&buffer).unwrap();
    // once it's decoded, we need to split the hash from the buffer
    // the hash should always be at the front of the buffer, and be
    // the same length since it's a SHA1 hash
    let hash = &data[..HASH_LENGTH];
    let hash_s = match str::from_utf8(hash) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    // confirm the hash
    let file_hash = hash_file(path, SHA1);
    if file_hash != hash_s {
        println!("Bad file hashes! Removing file...");
        println!("file_hash: {}\nhash_sfil: {}", file_hash, hash_s);
        fs::remove_file(path).unwrap();
    } else {
        println!("File was written to: {:?}", path);
    }
}
