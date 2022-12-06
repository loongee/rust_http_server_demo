use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(stream: TcpStream) {
    if let Ok(req) = handle_request(&stream) {
        println!("received request:\n{:#?}", req);
        handle_response(&stream, req);
    } else {
        println!("fail!");
    }
}

#[derive(Debug)]
struct HttpReq {
    method: String,
    path: String,
    protocol: String,
    headers: HashMap<String, String>,
}

impl HttpReq {
    fn parse_from_req(lines: Vec<String>) -> Result<HttpReq, String> {
        if lines.is_empty() {
            return Err(String::from("empty request!"));
        }

        let mut req_method = String::from("");
        let mut path = String::from("");
        let mut protocol = String::from("");
        if lines.first().is_some() {
            let parts = lines.first().unwrap().split(' ').collect::<Vec<&str>>();
            req_method = String::from(parts[0]);
            path = String::from(parts[1]);
            protocol = String::from(parts[2]);
        }

        let mut headers: HashMap<String, String> = HashMap::new();
        
        for line in &lines[1..] {
            let (key, value) = line.split_once(":").unwrap();
            headers.insert(String::from(key), String::from(value.trim()));
        }

        return Ok(HttpReq {
            method: req_method,
            path: path,
            protocol: protocol,
            headers: headers,
        });
    }
}

fn handle_request(mut stream: &TcpStream) -> Result<HttpReq, String> {
    let mut req_lines: Vec<String> = Vec::new();
    let mut line: String = String::new();
    'read: loop {
        let mut buffer = [0; 1024];
        let size = stream.read(&mut buffer).unwrap();
        line.push_str(std::str::from_utf8(&buffer[..size]).unwrap());

        loop {
            let break_index = line.find("\r\n");
            if break_index.is_some() {
                let (line_data, extra) = line.split_at(break_index.unwrap());
                if line_data.is_empty() {
                    break 'read;
                }
                req_lines.push(String::from(line_data));
                line = String::from(&extra[2..]);
            } else {
                break;
            }
        }
    }

    return HttpReq::parse_from_req(req_lines);
}

fn handle_response(mut stream: &TcpStream, req: HttpReq) {
    let ok_line = "HTTP/1.1 200 OK";
    let err_404_line = "HTTP/1.1 404 NOT FOUND";

    let (mut status_line, mut file_name) = if req.method != "GET" || req.path.contains("..") {
        (err_404_line, String::from("standard_resp/404.html"))
    } else {
        let mut path = std::env::current_dir().unwrap();
        path.push(req.path.trim_start_matches('/'));
        (ok_line, String::from(path.to_str().unwrap()))
    };

    let mut contents = fs::read_to_string(&file_name);
    if contents.is_err() {
        println!("read file:{file_name} failed!");
        (status_line, file_name) = (err_404_line, String::from("standard_resp/404.html"));
        contents = fs::read_to_string(file_name)
    }
    let response = format_http_response(status_line, &contents.unwrap());

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn format_http_response(status_line: &str, contents: &str) -> String {
    return format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
        );
}
