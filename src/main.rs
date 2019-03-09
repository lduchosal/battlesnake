extern crate tiny_http;

fn main() {
    use tiny_http::{Server, Response};

    let server = Server::http("0.0.0.0:6601").unwrap();
    let port = server.server_addr().port();
    println!("Now listening on port {}", port);


    let mut snake = 0;
    let config = "{\"color\":\"#ff00ff\",\"headType\":\"bendr\",\"tailType\":\"pixel\"}";
    let right = "{\"move\":\"right\"}";
    let left = "{\"move\":\"left\"}";
    let up = "{\"move\":\"up\"}";
    let down = "{\"move\":\"down\"}";
    let bye = "{\"say\":\"bye\"}";
    let hello = "{\"say\":\"hello\"}";
    let pong = "{\"say\":\"pong\"}";


    loop {

        println!("receving");

        let mut rq = match server.recv() {
            Ok(rq) => rq,
            Err(_) => break
        };

        println!(" rq method: {:?}", rq.method());
        println!(" rq url: {:?}", rq.url());
        println!(" rq headers: {:?}", rq.headers());
        println!(" rq body_length: {:?}", rq.body_length());

        let mut content = String::new();
        rq.as_reader().read_to_string(&mut content).unwrap();

        println!(" rq content: {:?}", content);

        let message = match rq.url() {
            "/start" => {
                snake = 0; 
                config
            },
            "/move" => {
                snake += 1;
                match snake % 4 {
                    0 => left,
                    1 => down,
                    2 => right,
                    3 => up,
                    _ => left
                }
            },
            "/end" => bye,
            "/ping" => pong,
            _ => hello
        };

        println!("response: {}", message);
        let response = Response::from_string(message);
        match rq.respond(response) {
            Ok(()) => {},
            Err(err) => println!("Error: {}", err),
        }
    }
}
