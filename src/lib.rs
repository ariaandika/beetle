#![allow(warnings)]

use std::{
    fs::{File, self},
    io::{BufReader, Write, Read, BufRead},
    net::{TcpListener, TcpStream, Shutdown}, process::exit, collections::HashMap, time::Duration, fmt::Display, path::Path,
};

type Callback = fn(req: Request, res: Respond);

enum HTTPMethod {
    Get(&'static str),
    Post(&'static str),
    Put(&'static str),
    Patch(&'static str),
    Delete(&'static str),
    Any(&'static str),
    No,
}

pub struct Server {
	get: Vec<(&'static str, Callback)>,
    post: Vec<(&'static str, Callback)>,
    put: Vec<(&'static str, Callback)>,
    patch: Vec<(&'static str, Callback)>,
    delete: Vec<(&'static str, Callback)>,
    serve_path: Vec<(String,String)>,
    notfound: String,
    timeout: u64,
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
    headers: Vec<(String, String)>,
}

impl Server {
    /// define endpoint of a get request
    pub fn get(&mut self, path: &'static str, cb: Callback) -> &mut Server { self.get.push((path, cb));self }
    
    /// define endpoint of a post request
    pub fn post(&mut self, path: &'static str, cb: Callback) -> &mut Server { self.post.push((path, cb));self }
    
    /// define endpoint of a put request
    pub fn put(&mut self, path: &'static str, cb: Callback) -> &mut Server { self.put.push((path, cb));self }
    
    /// define endpoint of a patch request
    pub fn patch(&mut self, path: &'static str, cb: Callback) -> &mut Server { self.patch.push((path, cb));self }
    
    /// define endpoint of a delete request
    pub fn delete(&mut self, path: &'static str, cb: Callback) -> &mut Server { self.delete.push((path, cb));self }
    
    /// define endpoint of any request method
    pub fn any(&mut self, path: &'static str, cb: Callback) -> &mut Server { self.delete.push((path, cb));self }
	
    /// create new server
    pub fn new() -> Self {
        Server {
            get: vec![], post: vec![], put: vec![], patch: vec![], delete: vec![],
            serve_path: vec![], notfound: "Not Found".to_string(), timeout: 30
        }
    }
	
    /// serve static files
    pub fn serve<P: AsRef<Path>>(&mut self, path: P) -> &mut Server {
		match Util::read_dir_recursive(path) {
			Ok(mut dirs) => self.serve_path.append(&mut dirs),
			Err(er) => Log::err(er),
		}
		self
	}
	
    /// finish setup and start listening
    pub fn listen(&mut self, port: u16) {
        let tcp = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap_or_else(|err|{
            Log::err(format!("Unable to start the server, detail:\n{err}"));
            exit(1);
        });
        
        for stream_res in tcp.incoming() {
			match Server::handle_stream_error(stream_res, self.timeout) {
				Ok(stream) => {
                    self.handler( stream );
                },
				Err(er) => Log::err(er),
			}
        }
    }
	
	fn handle_stream_error(stream: Result<TcpStream, std::io::Error>, tm: u64) -> Result<TcpStream,String>{
		let _stream = stream.map_err(|er|er.to_string())?;
		_stream
            .set_read_timeout(Some(Duration::new(tm, 0)))
            .map_err(|err|err.to_string())?;
		Ok(_stream)
	}
	
	fn read_buffer(stream: &mut TcpStream) -> (Vec<String>,HashMap<String,String>,Option<String>){
        let mut headers = HashMap::new();
		let mut reader = BufReader::new(stream);
		let mut meta;
		
		let mut line = String::new();
        
        if let Ok(size) = reader.read_line(&mut line) {
            if size != 0 {
                meta = line.clone().split_whitespace().map(|e|e.to_string()).collect();
                line.clear();
            } else { meta = vec![]; }
        } else { meta = vec![]; }
		
		while let Ok(size) = reader.read_line(&mut line) {
			if size == 0 { break } else if size == 2 { break }
			
			let (key,val) = line.split_once(": ").unwrap_or_default();
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

    fn handler(&mut self, mut stream: TcpStream) {
		
		// Read buffer
		let (meta,headers,body) = Server::read_buffer(&mut stream);
		
		// Prepare respond
		let mut res = Respond::new(stream);
		
		// if empty buffer, terminate
		if meta.len() <= 1 {
			println!("Empty Buffer");
			res.end(b"");
			return;
		}
		
        // collect request info
		let req = Request { path: meta[1].clone(), body, headers, method: meta[0].clone()  };
		let path = &req.path;
        let method = &req.method;
        
		print!("\x1b[92m{method}\x1b[39m {path} ");

		// static serve
		if !self.serve_path.is_empty() && method == &"GET" {
			match self.serve_path.iter().find(|_path|format!("/{}",_path.1).as_str() == path) {
				Some(file_dir) => {
                    let file_path = format!("{}/{}",file_dir.0,file_dir.1);
					match fs::read(&file_path) {
						Ok(file) => {
                            println!("\tStatic");
                            match Util::get_path_content_type(&file_path) {
                                Ok(ty) => res.set_header("Content-Type", ty),
                                Err(format) => Log::warn(format!("Content-Type {format} currently not listed"))
                            }
							return res.end(&file);
						},
						Err(er) => Log::err(er),
					}
				},
				None => {
                    // try serve index.html
                    let index_path = format!("{}{}index.html",path,if path.ends_with('/'){""}else{"/"} );
                    match self.serve_path.iter().find(|_path|format!("/{}",_path.1).as_str() == index_path.as_str()) {
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

        let method_check = match method.as_str() {
            "GET" => Some(&self.get),
            "POST" => Some(&self.post),
            "PUT" => Some(&self.put),
            "PATCH" => Some(&self.patch),
            "DELETE" => Some(&self.delete),
            _ => None,
        };
        
        match method_check {
            Some(method_endpoint) => {
                match method_endpoint.iter().find(|e| e.0 == path.to_string()) {
                    Some(endpoint) => {
				        println!("End point");
						endpoint.1(req, res);
					}
                    //* path not found
                    None => {
                        println!("\x1b[91mNot Found\x1b[0m");
                        res.status = ("Not Found".to_string(),404);
                        res.send(&self.notfound);
                    }
                }
            }
            None => {
                Log::warn(format!("method not found: {method}"));
                res.end(b"");
            }
        }
    }
}

impl Respond {
    pub fn new(stream: TcpStream) -> Self {
		Respond {
			stream,
			headers: vec![],
			respond_line: "".to_string(),
			status: (String::from("OK"),200), 
		}
	}
    
    //* ending request
    /// send and `end` request
    pub fn send(&mut self, content: &str) {
        self.end(content.as_bytes());
    }
    
    /// `end` and shutdown request
	pub fn end(&mut self, buf: &[u8]){
        self.write_header_to_stream(buf.len());   
        self.stream.write( buf ).unwrap_or_else(|err|{Log::err(format!("Failed to send response: {err}"));0});
        self.shutdown();
	}
    
    /// redirect and `end` the request
    pub fn redirect(&mut self, target: &str) {
        self.set_header("Location", target);
        self.set_status(304, "Redirect");
        self.end(b"");
    }
    
    /// stream file and `end` the stream
    pub fn file(&mut self, path: &str) -> std::io::Result<()> {
        let file = File::open(path)?;
		match Util::get_path_content_type(path) {
			Ok(ty) => self.set_header("Content-Type", ty),
			Err(format) => Log::warn(format!("Content-Type {format} currently not listed")),
		}
        self.write_header_to_stream((&file.metadata().unwrap()).len().try_into().unwrap());
        let mut reader = BufReader::new(file);
        let mut buffer = [0; 65536];   // 65536 is common file size for 64kb
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 { break; }
            self.stream.write_all(&buffer[..n])?;
        }
        Ok(self.shutdown())
    }

    //* response modifier
    /// add response headers
    pub fn set_header(&mut self, key: &str, val: &str) {
        self.headers.push((key.to_string(), val.to_string()));
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
    
    /// send image and `end` the request
    fn image(&mut self,path: &str){
        println!("191: >! Respond.image is under development");
        let file = match File::open(path) {
            Ok(content) => content,
            Err(_) => {
                self.set_status(404, "Not Found");
                self.send("");
                return;
            }
        };
		match Util::get_path_content_type(path) {
			Ok(ty) => self.set_header("Content-Type", ty),
			Err(format) => println!("Content-Type {format} currently not listed"),
		}
        
        self.write_header_to_stream((&file.metadata().unwrap()).len().try_into().unwrap());
        
        let mut buffer = BufReader::new(file);
        let mut buf_store = [0; 65536];   // 65536 is common file size for 64kb
        loop {
            let read = buffer.read(&mut buf_store).unwrap();
            if read == 0 { break; }
            self.stream.write_all(&buf_store[..read]).unwrap();
        }
        self.shutdown();
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
    
    /// send all available information
    fn debug(&mut self) {
        self.end(b""
			// self.request_lines.join("\r\n").as_bytes()
		);
    }
    
    //* inner utility
    /// write all header in request object
    fn write_header_to_stream(&mut self,conten_len: usize){
        self.headers.push(("Content-length".to_string(), conten_len.to_string()));
        match self.stream.write(
            format!("HTTP/1.1 {} {}\r\n{}\r\n\r\n",
            self.status.0,self.status.1,
            self.headers.iter().map(|e| format!("{}: {}", e.0, e.1)).collect::<Vec<String>>().join("\r\n")
        ).as_bytes()) {
            Ok(_) => (),
            Err(er) => Log::err(er),
        };
    }
    /// shutdown response
    fn shutdown(&mut self){
        // self.stream.shutdown(Shutdown::Both).unwrap();
        self.stream.flush().unwrap_or_else(|err|Log::err(err));
    }
}

pub enum Cors {
    AllowOrigin(String),
    AllowMethod(String),
    AllowHeader(String),
}

struct Util;
impl Util {
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
    fn get_path_content_type(path: &str) -> Result<&str,&str> {
        let format = path.split(".").last().unwrap_or_default();
        match Util::get_content_type(format) {
            Some(fmt) => Ok(fmt),
            None => Err(format),
        }
    }
    
    /// return (static folder, the rest of the path)[]
	fn read_dir_recursive<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<(String,String)>> {
		let mut flat_dirs = vec![];
		let res = fs::read_dir(path)?;
        
		for dir_entry_res in res {
			let dir_entry = dir_entry_res?;
			let file_type = dir_entry.file_type()?;
			
			if file_type.is_dir() {
				let mut flat_dir = Util::read_dir_recursive(&dir_entry.path())?;
				flat_dirs.append(&mut flat_dir);
			} else if file_type.is_file() {                
				let dirs_parse = dir_entry.path().to_str().unwrap_or("").replace("\\", "/");
				let dir_split = dirs_parse.split_once("/").unwrap_or(("",""));
				flat_dirs.push( (dir_split.0.to_string(),dir_split.1.to_string()) );
			}
		}
		Ok(flat_dirs)
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