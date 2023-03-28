#![allow(warnings)]
use std::{collections::HashMap, net::{TcpStream, TcpListener}, io::{BufReader, Write, Read, BufRead}, fs::{File, self}, path::{Path, Iter}, fmt::Display, default, process::exit, time::Duration};

type Callback = fn(req: Request, res: Respond);

pub struct Server {
    cb: Vec<(String, String, Callback)>,
    /// (folder, path, extension)
    serve_path: Vec<(String,String,Option<String>)>,
    timeout: u64,
    terminate: bool,
}

pub struct Request {
    pub path: String,
	pub headers: HashMap<String,String>,
	pub body: Option<String>,
	pub method: String,
}

pub struct Respond {
    pub stream: TcpStream,
	respond_line: String,
    status: (String,u16),
    headers: Vec<u8>,
}

impl Server {
    /// define endpoint of a get request
    pub fn get(&mut self, path: &str, cb: Callback) -> &mut Server { self.cb.push(("GET".to_string(), path.to_string(), cb));self }
    
    /// define endpoint of a post request
    pub fn post(&mut self, path: &str, cb: Callback) -> &mut Server { self.cb.push(("POST".to_string(), path.to_string(), cb));self }
    
    /// define endpoint of a put request
    pub fn put(&mut self, path: &str, cb: Callback) -> &mut Server { self.cb.push(("PUT".to_string(), path.to_string(), cb));self }
    
    /// define endpoint of a patch request
    pub fn patch(&mut self, path: &str, cb: Callback) -> &mut Server { self.cb.push(("PATCH".to_string(), path.to_string(), cb));self }
    
    /// define endpoint of a delete request
    pub fn delete(&mut self, path: &str, cb: Callback) -> &mut Server { self.cb.push(("DELETE".to_string(), path.to_string(), cb));self }
    
    /// define endpoint of any request method
    pub fn all(&mut self, path: &str, cb: Callback) -> &mut Server { self.cb.push(("ALL".to_string(), path.to_string(), cb));self }
    
    /// define endpoint of any request method
    pub fn any(&mut self, cb: Callback) -> &mut Server { self.cb.push(("ANY".to_string(), String::new(), cb));self }
    
    /// create new server
    pub fn new() -> Self { Self { cb: vec![], serve_path: vec![], timeout: 30, terminate: false } }
    
    /// terminate the server
    pub fn terminate(&mut self) {
        println!("Terminating...");
        self.terminate = true;
    }
    
    /// serve static files
    pub fn serve<P: AsRef<Path>>(&mut self, path: P) -> &mut Server {
		match Util::read_dir_recursive(path) {
			Ok(mut dirs) => self.serve_path.append(&mut dirs),
			Err(er) => Log::err(er),
		}
		self
	}
    
    /// finish setup and start listening and calledback when ready
    pub fn listen_cb(&mut self, port: u16, cb: fn()) {
        let tcp = TcpListener::bind(format!("127.0.0.1:{port}"))
        .unwrap_or_else(|err|{Log::err(format!("Unable to start the server, detail:\n{err}")); exit(1);});
    
        cb();
        for stream_res in tcp.incoming() {
			match Server::handle_stream_error(stream_res, self.timeout) {
				Ok(stream) => { self.handler( stream ); },
				Err(er) => Log::err(er),
			}
            if self.terminate { break }
        }
    }
    
    
    /// finish setup and start listening
    pub fn listen(&mut self, port: u16) {
        let tcp = TcpListener::bind(format!("127.0.0.1:{port}"))
        .unwrap_or_else(|err|{Log::err(format!("Unable to start the server, detail:\n{err}")); exit(1);});
    
        for stream_res in tcp.incoming() {
			match Server::handle_stream_error(stream_res, self.timeout) {
				Ok(stream) => { self.handler( stream ); },
				Err(er) => Log::err(er),
			}
            if self.terminate { break }
        }
    }
    
	fn handle_stream_error(mut stream: Result<TcpStream, std::io::Error>, tm: u64) -> std::io::Result<TcpStream>{
		let _stream = stream?;
		_stream.set_read_timeout(Some(Duration::new(tm, 0)))?;
		Ok(_stream)
	}
    
    /// return (meta[], headers{}, Option<body>)
    /// meta (method, path)
	fn read_buffer(stream: &mut TcpStream) -> (Vec<String>,HashMap<String,String>,Option<String>){
        let mut headers = HashMap::new();
		let mut reader = BufReader::new(stream);
		let mut meta = vec![];
		
		let mut line = String::new();
        
        if let Ok(size) = reader.read_line(&mut line) {
            if size != 0 {
                meta.extend(line.clone().split_whitespace().map(|e|e.to_string()));
                line.clear();
            }
        }
		
		while let Ok(size) = reader.read_line(&mut line) {
			if size == 0 { break } else if size == 2 { break }
			let (key, val) = line.split_once(": ").unwrap_or_default();
			headers.insert(key.to_string(), val.trim().to_string()).unwrap_or_default();
			line.clear();
		}
		
        // Read payload if `Content-Length` header found
		let body = headers.get("Content-Length").map(|header|{
            let len: usize = header.parse().unwrap_or_else(|er|{Log::err(format!("Failed parsing Content-Length header: {er}"));0});
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf).unwrap_or_default();
            String::from_utf8(buf).unwrap_or_else(|er|{Log::err(er);String::new()})
        });
		(meta, headers, body)
	}
    
    fn handler(&self, mut stream: TcpStream){
        
		let (meta,headers,body) = Server::read_buffer(&mut stream);
		let mut res = Respond::new(stream);
		
		// if empty buffer, terminate
		if meta.len() <= 1 {
			println!("Empty Buffer");
			res.end(b"");
			return;
		}
		
        // collect request info
		let req = Request { path: meta[1].clone(), body, headers, method: meta[0].clone()  };
        let method = &meta[0];
		let path = &meta[1];
		print!("\x1b[92m{method}\x1b[39m {path} ");
        
        
		// static serve
		if !self.serve_path.is_empty() && method == &"GET" {
			match self.serve_path.iter().find(|served|&format!("/{}",served.1) == path) {
				Some((folder, p, ext)) => {
                    let file_path = format!("{}/{}",folder,p);
					match fs::read(&file_path) {
						Ok(file) => {
                            println!("\tStatic");
                            if let Some(ty) = ext { res.set_header("Content-Type", ty); }
							return res.end(&file);
						},
						Err(er) => Log::err(er),
					}
				},
				None => {
                    // try serve index.html
                    let index_path = format!("{}{}index.html",path,if path.ends_with('/'){""}else{"/"} );
                    match self.serve_path.iter().find(|_path|format!("/{}",_path.1) == index_path) {
                        Some(file_dir) => {
                            let file_path = format!("{}/{}",file_dir.0,file_dir.1);
                            match fs::read(&file_path) {
                                Ok(file) => {
                                    println!("\tStatic");
                                    res.set_header("Content-Type", "text/html");
                                    res.end(&file);
                                    return
                                },
                                Err(er) => Log::err(er),
                            }
                        }
                        None => {/* No static file */}
                    }
                },
			}
		}

        
        
        for (_method,_path,cb) in self.cb.iter() {
            if _method == "ANY" { }
            else if _method == "ALL" && _path == path { }
            else if _method != method || _path != path { continue; }
            
            println!("End point");
            return cb(req,res)
        }
        
        println!("\x1b[91mNot Found\x1b[0m");
        res.status = ("Not Found".to_string(),404);
        res.send("<h1>404 Not Found</h1>");
    }
}

impl Respond {
    pub fn new(stream: TcpStream) -> Self {
		Respond {
			stream, headers: vec![],
            respond_line: "".to_string(), 
            status: (String::from("OK"),200)
		}
	}
    
    //* ending request
    /// send and `end` request
    pub fn send<S: AsRef<str>>(&mut self, content: S) {
        self.end(content.as_ref().as_bytes());
    }
    
    /// `end` and shutdown request
	pub fn end<B: AsRef<[u8]>>(&mut self, buf: B){
        let mut buffer: Vec<u8> = vec![];
        self.write_header(&mut buffer,buf.as_ref().len());
        buffer.extend( buf.as_ref().iter() );
        
        
        self.stream.write_all(&buffer).unwrap_or_else(|er|{Log::err(format!("Failed to send response: {er}"))});
        self.shutdown();
	}
    
    fn write_header(&mut self, buffer: &mut Vec<u8>, len: usize){
        buffer.extend( format!("HTTP/1.1 {} {}\r\n", self.status.0, self.status.1 ).bytes() );
        buffer.extend( format!("Content-Length: {}\r\n", len).bytes() );
        buffer.extend( self.headers.iter() );
        buffer.extend( "\r\n".bytes() );
    }
    
    fn shutdown(&mut self){
        // self.stream.shutdown(Shutdown::Both).unwrap();
        self.stream.flush().unwrap_or_else(Log::err);
    }
    
    pub fn stream_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()>{
		match Util::get_path_content_type(path.as_ref().to_str().unwrap_or_default()) {
			Ok(ty) => self.set_header("Content-Type", ty),
			Err(format) => Log::warn(format!("Format \"{format}\" currently not listed")),
		}
        let file = File::open(path)?;
        self.stream( BufReader::new(file) )
    }
    
    pub fn stream(&mut self, mut reader: BufReader<File>) -> std::io::Result<()>{
        let mut buffer = [0; 65536];   // 65536 is common file size for 64kb
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 { break; }
            self.stream.write_all(&buffer[..n])?;
        }
        Ok(self.shutdown())
    }

    /// redirect and `end` the request
    pub fn redirect<S: AsRef<str>>(&mut self, target: S) {
        self.set_header("Location", target.as_ref());
        self.set_status(304, "Redirect");
        self.end(b"");
    }
    
    //* response modifier
    /// add response headers
    pub fn set_header<S: AsRef<str>>(&mut self, key: S, val: S) {
        self.headers.extend( format!("{}: {}\r\n", key.as_ref(), val.as_ref()).bytes() )
    }
    
    /// set response cross origin request sharing
    pub fn set_cors(&mut self, cors: Cors) {
        match cors {
            Cors::AllowOrigin(e) => self.set_header("Access-Control-Allow-Origin", &e),
            Cors::AllowMethod(e) => self.set_header("Access-Control-Allow-Methods", &e),
            Cors::AllowHeader(e) => self.set_header("Access-Control-Allow-Headers", &e),
        }
    }
    
    /// set response status
    pub fn set_status(&mut self,status:u16,msg: &str){
        self.status = (msg.to_string(),status);
    }

    //* shortcut for common response
    fn render(&mut self, html: &str) {
        todo!();
        // self.set_headers("Content-Type", "text/html");
        // self.end(Parser::parse_html(fs::read_to_string(path).unwrap(),&self.params).as_bytes());
    }
    
    // TODO
    fn download(&mut self, path: &str) {
        todo!();// Content-Disposition: attachment; filename=quot.pdf;
        match Util::get_path_content_type(path) {
            Ok(ty) => self.set_header("Content-Type", ty),
            Err(_) => {},
        }
        self.set_header("Content-Disposition","attachement; filename=Rekap Presensi.csv");
    }
}

struct Log;
impl Log {
    fn err(msg: impl Display){
        println!("\x1b[91mError\x1b[0m: {msg}");
    }
    fn warn(msg: impl Display){
        println!("\x1b[93mWarn\x1b[0m: {msg}");
    }
}

pub enum Cors {
    AllowOrigin(String),
    AllowMethod(String),
    AllowHeader(String),
}

struct Util;
impl Util {
    fn get_path_content_type(path: &str) -> Result<&str,&str> {
        if path.len() == 0 { return Err(path) }
        let format = path.split(".").last().unwrap_or_default();
        match Util::get_content_type(format) {
            Some(fmt) => Ok(fmt),
            None => Err(format),
        }
    }
    
    /// return (static folder, the rest of the path, Content-Type)[]
	fn read_dir_recursive<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<(String,String,Option<String>)>> {
		let mut flat_dirs = vec![];
		let res = fs::read_dir(path)?;
        
		for dir_entry_res in res {
			let dir_entry = dir_entry_res?;
			let file_type = dir_entry.file_type()?;
			
			if file_type.is_dir() {
				let mut flat_dir = Util::read_dir_recursive(&dir_entry.path())?;
				flat_dirs.append(&mut flat_dir);
			} else if file_type.is_file() {                
				let dirs_parse = dir_entry.path().to_str().unwrap_or_default().replace("\\", "/");
                let p = dir_entry.path();
                let fmt = Util::get_content_type( p.extension().unwrap_or_default().to_str().unwrap_or_default() );
				let dir_split = dirs_parse.split_once("/").unwrap_or_default();
				flat_dirs.push( (dir_split.0.to_string(), dir_split.1.to_string(), fmt.map(|e|e.to_string())) );
			}
		}
		Ok(flat_dirs)
	}
    
    fn get_content_type(key: &str) -> Option<&str>{
        match key {
            /*
            multipart/mixed    
            multipart/alternative   
            multipart/related (using by MHTML (HTML mail).)  
            multipart/form-data
            */
            "js" => Some("application/javascript"),
            "cjs" => Some("application/javascript"),
            "json" => Some("application/json"),
            "pdf" => Some("application/pdf"),
            "zip" => Some("application/zip"),
            "urlencoded" => Some("application/x-www-form-urlencoded"),
            // img
            "gif" => Some("image/gif"),
            "jpg" => Some("image/jpeg"),
            "jpeg" => Some("image/jpeg"),
            "png" => Some("image/png"),
            "ico" => Some("image/vnd"),
            "svg" => Some("image/svg+xml"),
            "webp" => Some("image/webp"),
            "bmp" => Some("image/bmp"),
            // txt
            "css" => Some("text/css"),
            "csv" => Some("text/csv"),
            "html" => Some("text/html"),
            "plain" => Some("text/plain"),
            "txt" => Some("text/plain"),
            "xml" => Some("text/xml"),
            "csv" => Some("text/csv"),
            "rtf" => Some("text/rtf"),
            "md" => Some("text/markdown"),
            // vid
            "mpeg" => Some("video/mpeg"),
            "mp4" => Some("video/mp4"),
            _ => None,
        }
    }
}
pub trait Json {
    fn to_json(&self) -> String;
}
impl Json for Vec<String> {
    fn to_json(&self) -> String{
        let mut out = "[".to_string();
        self.iter().for_each(|v|{
            out.push_str(format!("\"{}\",",v).as_str());
        });
        out.pop();
        out.push(']');
        out
    }
}