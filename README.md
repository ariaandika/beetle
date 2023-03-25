# Rust TCPListener

## Getting started

```rs
mod server;

fn main() {
		// create new server
    let mut app = server::Server::new(4040);
    
		// add endpoint using builder pattern
    app
        .serve("view")
        .get("/",|_,mut res|{
            res.render("index.html");
        })
        .get("/app",|_,mut res|{
            res.send("<h1>Rust dev</h1>");
        })
        .post("/info", |req,mut res|{
            res.send(format!("<h1>Path : {}</h1>",req.path).as_str());
        });
    
		// listen
    app.listen(|p|println!("Listening in {p}..."))
}
```

## Dev note

when testing, the browser will send at least 3 request, main request, favicon, and the last request is an empty buffer. Web browsers send an additional empty request after the main request, which is known as an HTTP Keep-Alive request. HTTP Keep-Alive is a mechanism that allows a client to send multiple requests on a single TCP connection, without having to re-establish the connection for each request.

## Reference

- `type Callback = fn(req: Request, res: Respond)`

- `Server`
	- `new(port: u16) -> Self`
	- `listen(cb: fn(u16))`
	- `serve(path: String)`
	- `get(path: String, cb: Callback) -> Server`
	- `post(path: String, cb: Callback) -> Server`
	
- `Request`
	- `path: String`
	- `headers: HashMap`
	- `body: String`
	- `method: String`

- `Respond`
	- `send(content: &str)`
	- `add_headers(key: &str, val: &str)`
	- `set_cors(cors: Cors)`
	- `set_status(status:u16, msg: &str)`
	- `end(buf: &\[u8\])`
	- `redirect(target: &str)`
	- `file(path: &str)`
	- WIP
	- `add_params(param: Vec<(&str,&str)>)`
	- `render(view: &str)`
	- `download()`
	- `debug()`