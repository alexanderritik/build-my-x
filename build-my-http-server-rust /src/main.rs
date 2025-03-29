use std::net::TcpStream;
use std::{env, collections::HashMap, io::Read, io::Write, net::TcpListener};

use std::thread;
use std::fs;

fn extract_headers(buffer: [u8; 512]) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    let request_str = String::from_utf8_lossy(&buffer[..]);
    let mut splitted = request_str.split("\r\n");


    if let Some(status) = splitted.next() {
        println!("1 - {status}");
        let status_splitted: Vec<&str> = status.split(" ").collect();
        headers.insert("Type".to_string(), status_splitted[0].to_string());
        headers.insert("Route".to_string(), status_splitted[1].to_string());
        headers.insert("Version".to_string(), status_splitted[2].to_string());
    }
    println!("2 - {headers:?}");
    for split in splitted {
        let header_splitted: Vec<&str> = split.split(": ").collect();
        if header_splitted.len() >= 2 {
            println!("{:?} {:?}", header_splitted[1].to_string(), header_splitted[0].to_string());
            headers.insert(
                header_splitted[0].to_string(),
                header_splitted[1].to_string(),
            );
        }else if header_splitted[0].len() >= 1{
            let content = header_splitted[0].trim_end_matches('\0');
            println!("{:?}", content.to_string());
            headers.insert("Content".to_string(), content.to_string());
        }
    }
    println!("3 - {headers:?}");

    // println!("{:?}", splitted.next());
    headers
}
fn main() {
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                thread::spawn(move || concurrent_solution(&_stream));

            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn concurrent_solution(mut _stream: &TcpStream) {
    let mut buffer = [0; 512];
    _stream.read(&mut buffer).unwrap();
    let headers = extract_headers(buffer);
    let mut response: String = String::new();
    if let (Some(type_value), Some(route_value), Some(content_value)) =
        (headers.get("Type"), headers.get("Route") , headers.get("Content"))
    {
        if type_value == "GET" && route_value == "/" {
            response = "HTTP/1.1 200 OK\r\n\r\n".to_string();
        } else if type_value == "GET" && route_value.starts_with("/echo/") {
            let splitted: Vec<&str> = route_value.split("/").collect();
            let param = splitted[2];
            let length = param.len();
            response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", length, param);
        } else if type_value == "GET" && route_value.starts_with("/files/") {
            let splitted: Vec<&str> = route_value.split("/").collect();
            let env_args: Vec<String> = env::args().collect();
            let param = splitted[2];
            let mut dir = env_args[2].clone();
            dir.push_str(param);
            match fs::read(dir) {
                Ok(file_content) => {
                    response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                        file_content.len(),
                        String::from_utf8(file_content).expect("No content")
                    );
                }
                Err(_) => {
                    response = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
                }
            }
        } else if type_value == "POST" && route_value.starts_with("/files/") {
            let splitted: Vec<&str> = route_value.split("/").collect();
            let env_args: Vec<String> = env::args().collect();
            let param = splitted[2];
            let mut dir = env_args[2].clone();
            dir.push_str(param);
            fs::write(dir, content_value);
            response = "HTTP/1.1 201 Created\r\n\r\n".to_string();
        } else if type_value == "GET" && route_value.starts_with("/user-agent") {
            if let Some(user_agent) = headers.get("User-Agent") {
                response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", user_agent.len(), user_agent);
            }
        } else {
            response = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
        }
    }
    println!("{}", response);
    _stream.write_all(response.as_bytes()).unwrap();
}