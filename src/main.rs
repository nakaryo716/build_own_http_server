use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

use build_own_http_server::ThreadPool;

fn main() {
    let listener = std::net::TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(4);

    println!("Server is running: http://localhost:8080");

    loop {
        let (stream, _) = listener.accept().unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_read = BufReader::new(&stream);
    let http_req: Vec<_> = buf_read
        .lines()
        // this error will happen when the text is not utf-8
        .map(|result| result.unwrap())
        .take_while(|v| !v.is_empty())
        .collect();

    let (status_line, file_name) = match http_req[0].as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(file_name).unwrap();
    let contents_len = contents.len();

    let res = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line, contents_len, contents
    );
    stream.write_all(res.as_bytes()).unwrap();
}
