extern crate tiny_http;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use serde_json::Result;

fn main() {
    use tiny_http::{Server, Response};

    let server = Server::http("0.0.0.0:6601").unwrap();
    let port = server.server_addr().port();
    println!("Now listening on port {}", port);

    let config = Config {
        color: "#ff00ff".to_string(),
        head: "bendr".to_string(),
        tail: "pixel".to_string()
        };

    let right = Move { dir: "right".to_string() };
    let left = Move { dir: "left".to_string() }; 
    let up = Move { dir: "up".to_string() }; 
    let down = Move { dir: "down".to_string() };

    let bye = Say { say: "bye".to_string() };
    let hello = Say { say: "hello".to_string() };
    let pong = Say { say: "pong".to_string() };

    let sconfig = serde_json::to_string(&config).unwrap();

    let sright = serde_json::to_string(&right).unwrap();
    let sleft = serde_json::to_string(&left).unwrap();
    let sup = serde_json::to_string(&up).unwrap();
    let sdown = serde_json::to_string(&down).unwrap();

    let sbye = serde_json::to_string(&bye).unwrap();
    let shello = serde_json::to_string(&hello).unwrap();
    let spong = serde_json::to_string(&pong).unwrap();


    loop {

        println!("receving");

        let mut rq = match server.recv() {
            Ok(rq) => rq,
            Err(_) => break
        };

        println!(" rq method: {:?}", rq.method());
        println!(" rq url: {:?}", rq.url());
        // println!(" rq headers: {:?}", rq.headers());
        println!(" rq body_length: {:?}", rq.body_length());

        let mut content = String::new();
        rq.as_reader().read_to_string(&mut content).unwrap();

        println!(" rq content: {:?}", content);
        let g: Option<Game> = match serde_json::from_str(content.as_ref()) {
            Ok(g) => Some(g),
            Err(_) => None,
        };

        let message = match rq.url() {
            "/start" => sconfig.clone(),
            "/move" => sdown.clone(),
            "/end" => sbye.clone(),
            "/ping" => spong.clone(),
            _ => shello.clone()
        };

        println!("response: {}", message);
        let response = Response::from_string(message);
        match rq.respond(response) {
            Ok(()) => {},
            Err(err) => println!("Error: {}", err),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "color")]
    color: String,

    #[serde(rename = "headType")]
    head: String,

    #[serde(rename = "tailType")]
    tail: String,
}

#[derive(Serialize, Deserialize)]
pub struct Move {
    #[serde(rename = "move")]
    dir: String,
}

#[derive(Serialize, Deserialize)]
pub struct Say {
    #[serde(rename = "say")]
    say: String,
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    #[serde(rename = "game")]
    game: GameId,

    #[serde(rename = "turn")]
    turn: i64,

    #[serde(rename = "board")]
    board: Board,

    #[serde(rename = "you")]
    you: Snake,
}

#[derive(Serialize, Deserialize)]
pub struct Board {
    #[serde(rename = "height")]
    height: i64,

    #[serde(rename = "width")]
    width: i64,

    #[serde(rename = "food")]
    food: Vec<Food>,

    #[serde(rename = "snakes")]
    snakes: Vec<Snake>,
}

#[derive(Serialize, Deserialize)]
pub struct Food {
    #[serde(rename = "x")]
    x: i64,

    #[serde(rename = "y")]
    y: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Snake {
    #[serde(rename = "id")]
    id: String,

    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "health")]
    health: i64,

    #[serde(rename = "body")]
    body: Vec<Food>,
}

#[derive(Serialize, Deserialize)]
pub struct GameId {
    #[serde(rename = "id")]
    id: String,
}

