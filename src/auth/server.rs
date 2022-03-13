use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
use std::time::Duration;

use http::Uri;
use httparse;


pub fn oauth_server(port: u16) -> String {
    // Bind free port from range [15000, 25000]
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

    let mut result: String = String::new();

    println!("Starting server");
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        match handle_oauth_request(stream) {
            Ok(t) => {
                result = t;
                break;
            }
            Err(_) => continue
        };
    };
    println!("Stopped server [{}]", result);
    return result;
}

fn vec_to_hashmap(v: Vec<(String, String)>) -> HashMap<String, String> {
    let map = HashMap::from_iter(v.into_iter());
    map
}

// Parse string like "key1=value1&key2=value2&key3=value3" to [(key1, value1), (key2, value2), ...]
fn get_params(path: &str) -> Vec<(String, String)> {
    let mut resp: Vec<(String, String)> = vec![];
    for i in path.split("&").collect::<Vec<&str>>() {
        let v = i.split("=").collect::<Vec<&str>>();
        resp.push((v[0].to_string(), v[1].to_string()));
    };
    resp
}

fn handle_oauth_request(mut stream: TcpStream) -> Result<String, Box<dyn Error>> {
    // The part we need cannot be longer than 1024 u8-chars
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    // This section need for HTTP request parsing
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    let res = req.parse(&buffer).unwrap();

    let code: String;

    // We don't need complete response
    if res.is_partial() {
        match req.path {
            Some(path) => {
                let uri = path.parse::<Uri>().unwrap();
                let query = uri.query().unwrap();
                let params = vec_to_hashmap(get_params(query));
                code = params["code"].clone();
            }
            None => {
                panic!("Empty path");
            }
        }
    } else {
        panic!("Incorrect request {:?}", res)
    }

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        code.len(),
        format!("Success {}", code)
    );

    // Send response
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    sleep(Duration::from_secs(1));  // Let the response be sent to client

    Ok(code)
}