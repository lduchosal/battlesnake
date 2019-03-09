extern crate tiny_http;
extern crate serde_json;

use serde::{Deserialize, Serialize};

fn main() {
    use tiny_http::{Server, Response};

    let server = Server::http("0.0.0.0:6601").unwrap();
    let port = server.server_addr().port();
    println!("Now listening on port {}", port);

    let config = Config {
        color: "#fe00fe".to_string(),
        head: HeadType::SandWorm,
        tail: TailType::BlockBum
        };

    let bye = Say::Bye;
    let hello = Say::Hello;
    let pong = Say::Pong;

    let sconfig = serde_json::to_string(&config).unwrap();

    let sbye = serde_json::to_string(&bye).unwrap();
    let shello = serde_json::to_string(&hello).unwrap();
    let spong = serde_json::to_string(&pong).unwrap();


    loop {

        println!("--");

        let mut rq = match server.recv() {
            Ok(rq) => rq,
            Err(_) => break
        };

        // println!(" rq method: {:?}", rq.method());
        println!(" rq url: {:?}", rq.url());
        // println!(" rq headers: {:?}", rq.headers());
        // println!(" rq body_length: {:?}", rq.body_length());

        let mut content = String::new();
        rq.as_reader().read_to_string(&mut content).unwrap();

        println!(" rq content: {:?}", content);

        let mut result: String;

        let message = match rq.url() {
            "/start" => &sconfig,
            "/move" => {
                
                let next_move = play(content.as_ref());
                result = serde_json::to_string(&next_move).unwrap();
                &result
            },
            "/end" => &sbye,
            "/ping" => &spong,
            _ => &shello
        };

        println!("message: {}", message);
        let response = Response::from_string(message.clone());
        match rq.respond(response) {
            Ok(()) => {},
            Err(err) => println!("Error: {}", err),
        }
    }
}

fn play(content: &str) -> Move {

    let g: Game = serde_json::from_str(content).expect("Unable to deserialize game");

    let mut possibles = possibles(&g.you);
    check_walls(&g, &mut possibles);
    check_snakes(&g, &mut possibles);
    check_collisions(&g, &mut possibles);
    dump_results(&possibles);

    possibles.sort_by(|a, b| b.value.cmp(&a.value));
    let bestfit = possibles.first().unwrap();
    bestfit.dir.clone()
}

fn possibles(snake: &Snake) -> Vec<Possible> {
    let head = &snake.body[0];

    let mut possibles: Vec<Possible> = Vec::new();
    let up = Possible { point: Point { x: head.x, y: head.y -1 }, dir: Move::Up, value: 10 };
    let down = Possible { point: Point { x: head.x, y: head.y +1 }, dir: Move::Down, value: 10 };
    let left = Possible { point: Point { x: head.x -1, y: head.y }, dir: Move::Left, value: 10 };
    let right = Possible { point: Point { x: head.x +1, y: head.y }, dir: Move::Right, value: 10 };

    possibles.push(up);
    possibles.push(down);
    possibles.push(left);
    possibles.push(right);

    possibles
}

fn dump_results(possibles: &Vec<Possible>) {
    for p in possibles {
        println!("possible: {:?}", p);
    }
}

fn check_walls(game: &Game, possibles: &mut Vec<Possible>) {

    for p in possibles {
        if p.point.x < 0 {
            p.value -= 10;
        }
        if p.point.y < 0 {
            p.value -= 10;
        }
        if p.point.x >= (game.board.width as i8) {
            p.value -= 10;
        }
        if p.point.y >= (game.board.height as i8) {
            p.value -= 10;
        }
    }
}

fn check_snakes(game: &Game, possibles: &mut Vec<Possible>) {

    for p in possibles {
        for s in &game.board.snakes {
            for b in &s.body {
                if p.point == *b {
                    p.value -= 10 ;
                }
            }
        }
    }
}

fn check_collisions(game: &Game, ps: &mut Vec<Possible>) {

    for p in ps {
        for s in &game.board.snakes {

            let pothers = possibles(&s);
            for po in pothers {
                if p.point == po.point {
                    p.value -= 1 ;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Possible {
    point: Point,
    dir: Move,
    value: i32
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "color")]
    color: String,

    #[serde(rename = "headType")]
    head: HeadType,

    #[serde(rename = "tailType")]
    tail: TailType,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum HeadType {
    Beluga,
    Bendr,
    Dead,
    Evil,
    Fang,
    Pixel,
    Regular,
    Safe,
    SandWorm,
    Shades,
    Silly,
    Smile,
    Tongue,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TailType {
    BlockBum,
    Bolt,
    Curled,
    FatRattle,
    Freckled,
    Hook,
    Pixel,
    Regular,
    RoundBum,
    Sharp,
    Skinny,
    SmallRattle
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "move")]
enum Move {
    Up,
    Down,
    Left,
    Right
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "say")]
enum Say {
    Bye,
    Hello,
    Pong
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    #[serde(rename = "game")]
    game: GameId,

    #[serde(rename = "turn")]
    turn: u32,

    #[serde(rename = "board")]
    board: Board,

    #[serde(rename = "you")]
    you: Snake,
}

#[derive(Serialize, Deserialize)]
pub struct Board {
    #[serde(rename = "height")]
    height: u8,

    #[serde(rename = "width")]
    width: u8,

    #[serde(rename = "food")]
    food: Vec<Point>,

    #[serde(rename = "snakes")]
    snakes: Vec<Snake>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Point {
    #[serde(rename = "x")]
    pub x: i8,

    #[serde(rename = "y")]
    pub y: i8,
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && 
        self.y == other.y
    }
}

#[derive(Serialize, Deserialize)]
pub struct Snake {
    #[serde(rename = "id")]
    id: String,

    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "health")]
    health: u8,

    #[serde(rename = "body")]
    body: Vec<Point>,
}

#[derive(Serialize, Deserialize)]
pub struct GameId {
    #[serde(rename = "id")]
    id: String,
}

