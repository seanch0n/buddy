use base64::{decode, encode};
use clap::{App, Arg};
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

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

    let file_path = "myfile.txt";
    if matches.is_present("client") {
        send_file(file_path.to_string());
    } else if matches.is_present("server") {
        println!("recv_file is getting called!");
        recv_file(file_path.to_string());
    }
}

fn send_file(file_path: String) -> io::Result<()> {
    let mut file = File::open(file_path)?;
    let mut tcp = TcpStream::connect("127.0.0.1:4321")?;
    let mut buf = [0; 4096];
    println!("We are sending the file!");
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            // reached end of file
            break;
        }
        // base64 encode the data before sending for integrity
        let encoded_buffer = base64::encode(buf);
        tcp.write_all(&encoded_buffer[..n].as_bytes())?;
    }
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
    let mut buffer = [0; 4096];

    stream.read(&mut buffer).unwrap();
    let decoded_buffer = base64::decode(&buffer).unwrap();

    fs::write("blah.txt", decoded_buffer).expect("Unable to write file");
}
