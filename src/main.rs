extern crate tiny_http;
extern crate serde_json;
extern crate rand;

// import commonly used items from the prelude:
use rand::prelude::*;

use serde::{Deserialize, Serialize};

fn main() {
    use tiny_http::{Server, Response};

    // let server = Server::http("0.0.0.0:6605").unwrap();
    let server = Server::http("0.0.0.0:6600").unwrap();
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

    let head = &g.you.body[0];

    let mut possibles = possibles(&head);
    check_walls(&g, &mut possibles);
    check_snakes(&g, &mut possibles);
    check_tails(&g, &mut possibles);
    check_heads(&g, &mut possibles);
    kill_heads(&g, &mut possibles);
    hit_or_leave(&g, &mut possibles);
    prefer_food(&g, &mut possibles);
    look_for_tail(&g, &mut possibles);
    forward_thinking(&g, &mut possibles, 8 );

    dump_results(&possibles);
    best_fit(&mut possibles)

}

fn best_fit(possibles: &mut Vec<Possible>) -> Move {

    possibles.sort_by(|a, b| 
        b.value
            .cmp(&a.value)
            .then(
                b.rand.cmp(&a.rand)
            )
    );
    let bestfit = possibles.first().unwrap();
    bestfit.dir.clone()
}

fn possibles(head: &Point) -> Vec<Possible> {

    let mut possibles: Vec<Possible> = Vec::new();
    let up = Possible::new( head.x, head.y -1, Move::Up);
    let down = Possible::new( head.x, head.y +1, Move::Down);
    let left = Possible::new( head.x -1, head.y, Move::Left);
    let right = Possible::new( head.x +1, head.y, Move::Right);

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

        let mut check_walls = 0;

        if p.point.x < 0 {
            check_walls -= 10;
        }
        if p.point.y < 0 {
            check_walls -= 10;
        }
        if p.point.x >= (game.board.width as i16) {
            check_walls -= 10;
        }
        if p.point.y >= (game.board.height as i16) {
            check_walls -= 10;
        }

        p.value += check_walls;
        p.check_walls = check_walls;
    }

}

fn prefer_food(game: &Game, possibles: &mut Vec<Possible>) {


    for p in possibles {

        let mut closest: i32 = game.board.height as i32;
        for f in &game.board.food {

            let distancex = (p.point.x as i32 - f.x as i32).abs();
            let distancey = (p.point.y as i32 - f.y as i32).abs();

            let distance = distancex + distancey;

            if distance < closest {
                closest = distance;
            }
        }

        let value = 5 - closest;

        p.value += value;
        p.prefer_food = value;
    }
}

fn look_for_tail(game: &Game, possibles: &mut Vec<Possible>) {

    let body = &game.you.body;
    if body.len() < 3 {
        return;
    }

    if game.you.health < 50 {
        return;
    }

    let tail = body.last().unwrap();
    let mut closest: i32 = game.board.height as i32 * 2;
    for p in possibles {


        let distancex = (p.point.x as i32 - tail.x as i32).abs();
        let distancey = (p.point.y as i32 - tail.y as i32).abs();
        let total = distancex + distancey;

        if total < closest {
            closest = total;
        }

        let value = 5 - closest;
        p.value += value;
        p.look_for_tail = value;
    }
}

fn check_snakes(game: &Game, possibles: &mut Vec<Possible>) {

    let check_snakes = 10;

    for p in possibles {
        for s in &game.board.snakes {
            for b in &s.body {
                if p.point == *b {
                    p.value -= check_snakes ;
                    p.check_snakes -= check_snakes;
                }
            }
        }
    }
}

fn check_tails(game: &Game, possibles: &mut Vec<Possible>) {

    let check_tails = 10;

    for p in possibles {
        for s in &game.board.snakes {
            match s.body.last() {
                None => {},
                Some(tail) => {
                    if p.point == *tail {
                        p.value += check_tails;
                        p.check_tails = check_tails;
                    }
                }
            }
        }
    }
}

fn check_heads(game: &Game, ps: &mut Vec<Possible>) {

    let check_heads = 1;
    for p in ps {
        for s in &game.board.snakes {

            let head = &s.body[0];
            let pothers = possibles(head);
            for po in pothers {
                if p.point == po.point {
                    p.value -= check_heads;
                    p.check_heads = check_heads;
                }
            }
        }
    }
}

fn kill_heads(game: &Game, ps: &mut Vec<Possible>) {

    let kill_heads = 8;
    for p in ps {
        for s in &game.board.snakes {

            let head = &s.body[0];
            let pothers = possibles(head);
            for po in pothers {
                if p.point != po.point {
                    continue;
                }
                if s.body.len() < game.you.body.len() {
                    p.value += kill_heads;
                    p.kill_heads += kill_heads;
                }

            }
        }
    }
}

fn hit_or_leave(game: &Game, ps: &mut Vec<Possible>) {

    for p in ps {
        for s in &game.board.snakes {

            if s.id == game.you.id {
                continue;
            }

            let head = &s.body[0];
            let pothers = possibles(head);

            let mut value = 5;
            if s.body.len() >= game.you.body.len() {
                value *= -1;
            }

            for po in pothers {
                if p.point == po.point {
                    p.value += value ;
                    p.hit_or_leave += value ;
                }
            }
        }
    }
}

fn forward_thinking(game: &Game, ps: &mut Vec<Possible>, depth: u8) {

    for p in ps {

        let mut futur = game.clone();
        let mut pathes = Vec::<Path>::new();

        let root = Path { point: p.point.clone(), level: 0 };
        pathes.push(root);

        for level in 0..depth {


            for snake in &mut futur.board.snakes {
                snake.body.pop();
            }
            futur.you.body.pop();

            let level_pathes = find_pathes(&pathes, level);
            for path in &level_pathes {

                let mut fps = possibles(&path.point);
                check_walls(&futur, &mut fps);
                check_snakes(&futur, &mut fps);

                for fp in fps {
                    if fp.value > 0 {
                        let path = Path { point: fp.point.clone(), level: level +1 };
                        pathes.push(path);
                    }
                }

            }

            for snake in &mut futur.board.snakes {
                if &snake.id == &futur.you.id {

                    for path in &level_pathes {
                        snake.body.push(path.point.clone());
                    }
                    futur.you = snake.clone();
                }
            }

        }

        let forward_thinking =  10 ;
        p.forward_pathes_len = pathes.len() as i32;

        if pathes.len() < depth as usize {
            p.value -= forward_thinking;
            p.forward_thinking -= forward_thinking;
            p.forward_pathes = pathes;
        }
    }
}


fn find_pathes(pathes: &Vec::<Path>, level: u8) -> Vec::<Path> {
    let mut result = Vec::<Path>::new();

    for path in pathes {
        if path.level == level {
            let chosen = path.clone();
            result.push(chosen);
        }
    }

    result
}

#[derive(Debug)]
pub struct Possible {
    point: Point,
    dir: Move,
    value: i32,
    check_walls: i32,
    check_snakes: i32,
    check_tails: i32,
    check_heads: i32,
    kill_heads: i32,
    prefer_food: i32,
    hit_or_leave: i32,
    look_for_tail: i32,
    forward_thinking: i32,
    forward_pathes: Vec<Path>,
    forward_pathes_len: i32,

    rand: u8
}

impl Possible {
    fn new(x: i16, y: i16, dir: Move) -> Possible {
        Possible { 
            point: Point { x: x, y: y }, 
            dir: dir, 
            value: 10, 
            rand: random(),
            check_walls: 0,
            check_snakes: 0,
            check_tails: 0,
            check_heads: 0,
            prefer_food: 0,
            hit_or_leave: 0,
            look_for_tail: 0,
            forward_thinking: 0,
            kill_heads: 0,
            forward_pathes: Vec::new(),
            forward_pathes_len: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    point: Point,
    level: u8,
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Board {
    #[serde(rename = "height")]
    height: u16,

    #[serde(rename = "width")]
    width: u16,

    #[serde(rename = "food")]
    food: Vec<Point>,

    #[serde(rename = "snakes")]
    snakes: Vec<Snake>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Point {
    #[serde(rename = "x")]
    pub x: i16,

    #[serde(rename = "y")]
    pub y: i16,
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && 
        self.y == other.y
    }
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct GameId {
    #[serde(rename = "id")]
    id: String,
}



#[test]
fn test() {

    let game = "{\"game\":{\"id\":\"36aab8cc-ee81-48e7-94f3-fe2e27c92abe\"},\"turn\":78,\"board\":{\"height\":11,\"width\":11,\"food\":[{\"x\":7,\"y\":0},{\"x\":9,\"y\":2},{\"x\":1,\"y\":1}],\"snakes\":[{\"id\":\"gs_WgVH9DYDyTDyJ3SvYMDtgWwB\",\"name\":\"lduchosal / charlesmanson-dev-online\",\"health\":98,\"body\":[{\"x\":0,\"y\":6},{\"x\":1,\"y\":6},{\"x\":2,\"y\":6},{\"x\":3,\"y\":6},{\"x\":4,\"y\":6},{\"x\":5,\"y\":6},{\"x\":6,\"y\":6},{\"x\":7,\"y\":6}]}]},\"you\":{\"id\":\"gs_WgVH9DYDyTDyJ3SvYMDtgWwB\",\"name\":\"lduchosal / charlesmanson-dev-online\",\"health\":98,\"body\":[{\"x\":0,\"y\":6},{\"x\":1,\"y\":6},{\"x\":2,\"y\":6},{\"x\":3,\"y\":6},{\"x\":4,\"y\":6},{\"x\":5,\"y\":6},{\"x\":6,\"y\":6},{\"x\":7,\"y\":6}]}}";

    play(game);

}
