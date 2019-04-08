#![feature(try_trait)]
extern crate time;

extern crate tiny_http;
extern crate serde_json;
extern crate rand;
extern crate indextree;

use indextree::Arena;
use indextree::NodeId;

use time::PreciseTime;


use rand::prelude::*;
use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

fn main() {
    use tiny_http::{Server, Response};

    let home = "6601";
    let train = "6600";
    let laurabush = "6615";

    let server = Server::http(format!("0.0.0.0:{}", laurabush)).unwrap();

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

    let result = engine(content);

    result
}

fn engine(content: &str) -> Move {

    let g: Game = serde_json::from_str(content).expect("Unable to deserialize game");

    let start = PreciseTime::now();

    let head = &g.you.body[0];

    let mut possibles = possibles(&head);
    let tpossibles = PreciseTime::now();

    check_walls(&g, &mut possibles);
    let tcheck_walls = PreciseTime::now();

    check_snakes(&g, &mut possibles);
    let tcheck_snakes = PreciseTime::now();

    check_tails(&g, &mut possibles);
    let tcheck_tails = PreciseTime::now();

    check_heads(&g, &mut possibles);
    let tcheck_heads = PreciseTime::now();

    kill_heads(&g, &mut possibles);
    let tkill_heads = PreciseTime::now();

    hit_or_leave(&g, &mut possibles);
    let thit_or_leave = PreciseTime::now();

    prefer_food(&g, &mut possibles);
    let tprefer_food = PreciseTime::now();

    prefer_food_distance(&g, &mut possibles);
    let tprefer_food_distance = PreciseTime::now();

    eat_my_food(&g, &mut possibles);
    let teat_my_food = PreciseTime::now();

    look_for_tail(&g, &mut possibles);
    let tlook_for_tail = PreciseTime::now();

    forward_thinking(&g, &mut possibles, 30);
    let tforward_thinking = PreciseTime::now();

    prefer_forward_space(&g, &mut possibles);
    let tprefer_forward_space = PreciseTime::now();

    hunt_snakes(&g, &mut possibles);
    let thunt_snakes = PreciseTime::now();

    let mut futur = build_futur(&g);
    let tbuild_futur = PreciseTime::now();

    longest_futur(&mut futur);
    let tlongest_futur = PreciseTime::now();

    enroule_ton_snake(&g, &mut possibles);
    let tenroule_ton_snake = PreciseTime::now();
    // (&g, &mut possibles);

    dump_results(&possibles);
    let tdump_results = PreciseTime::now();

    let bestfit = best_fit(&mut possibles);
    let tbestfit = PreciseTime::now();

    let timings = vec![
        ("start                   ", start                   ),
        ("possible                ", tpossibles              ),
        ("check_walls             ", tcheck_walls            ),
        ("check_snakes            ", tcheck_snakes           ),
        ("check_tails             ", tcheck_tails            ),
        ("check_heads             ", tcheck_heads            ),
        ("kill_heads              ", tkill_heads             ),
        ("hit_or_leave            ", thit_or_leave           ),
        ("prefer_food             ", tprefer_food            ),
        ("prefer_food_distance    ", tprefer_food_distance   ),
        ("eat_my_food             ", teat_my_food            ),
        ("look_for_tail           ", tlook_for_tail          ),
        ("forward_thinking        ", tforward_thinking       ),
        ("prefer_forward_space    ", tprefer_forward_space   ),
        ("hunt_snakes             ", thunt_snakes            ),
        ("build_futur             ", tbuild_futur            ),
        ("longest_futur           ", tlongest_futur          ),
        ("enroule_ton_snake       ", tenroule_ton_snake      ),
        ("dump_results            ", tdump_results           ),
        ("bestfit                 ", tbestfit                ),
    ];

    println!("");
    print_timing(timings);
    println!("");

    bestfit
}

fn longest_futur(futur: &mut Arena<Point>) {

    let mut pathes = convert_pathes(futur);
    pathes.sort_by(|v,w| v.len().cmp(&w.len()));
    print_pathes(&pathes);
}

fn print_timing(timings: Vec<(&str, PreciseTime)>) {

    for i in 0..timings.len()-1 {

        let start = timings[i];
        let end = timings[i+1];

        println!("{} ms {}.", start.1.to(end.1).num_milliseconds(), end.0);
    }

    let start = timings[0];
    let end = timings[timings.len()-1];
    println!("{} ms total.", start.1.to(end.1).num_milliseconds());

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

    let wall = 100;
    for p in possibles {

        let mut check_walls = 0;

        if p.point.x < 0 {
            check_walls -= wall;
        }
        if p.point.y < 0 {
            check_walls -= wall;
        }
        if p.point.x >= (game.board.width as i16) {
            check_walls -= wall;
        }
        if p.point.y >= (game.board.height as i16) {
            check_walls -= wall;
        }

        p.value += check_walls;
        p.check_walls = check_walls;
    }

}

#[test]
fn test_mange_la_pomme() {

    let game = "{\"game\":{\"id\":\"91ebc415-da16-4c8b-b8aa-feedb74f0040\"},\"turn\":88,\"board\":{\"height\":11,\"width\":11,\"food\":[{\"x\":3,\"y\":0}],\"snakes\":[{\"id\":\"gs_G73RbQqT94r3fkMwPXpYYQWX\",\"name\":\"xm-evanguo / pinkpinkpenguin\",\"health\":96,\"body\":[{\"x\":8,\"y\":8},{\"x\":8,\"y\":9},{\"x\":9,\"y\":9},{\"x\":9,\"y\":8},{\"x\":10,\"y\":8},{\"x\":10,\"y\":7},{\"x\":9,\"y\":7},{\"x\":8,\"y\":7}]},{\"id\":\"gs_j6TBPGM74PyrxjHk6FMf4xmW\",\"name\":\"robbles / ROB 2018\",\"health\":75,\"body\":[{\"x\":3,\"y\":9},{\"x\":3,\"y\":10},{\"x\":4,\"y\":10},{\"x\":5,\"y\":10},{\"x\":6,\"y\":10},{\"x\":7,\"y\":10},{\"x\":7,\"y\":9},{\"x\":6,\"y\":9}]},{\"id\":\"gs_hMxC94hH8qtJbbFMXWJ9fc8M\",\"name\":\"neovas / hissin-bastid\",\"health\":77,\"body\":[{\"x\":7,\"y\":5},{\"x\":8,\"y\":5},{\"x\":8,\"y\":4},{\"x\":7,\"y\":4}]},{\"id\":\"gs_gR4MvwQJMBMBfBfYm7pdmp7S\",\"name\":\"lduchosal / boblee-0.13\",\"health\":97,\"body\":[{\"x\":3,\"y\":3},{\"x\":3,\"y\":4},{\"x\":3,\"y\":5},{\"x\":3,\"y\":6},{\"x\":4,\"y\":6},{\"x\":4,\"y\":5},{\"x\":4,\"y\":4}]}]},\"you\":{\"id\":\"gs_gR4MvwQJMBMBfBfYm7pdmp7S\",\"name\":\"lduchosal / boblee-0.13\",\"health\":97,\"body\":[{\"x\":3,\"y\":3},{\"x\":3,\"y\":4},{\"x\":3,\"y\":5},{\"x\":3,\"y\":6},{\"x\":4,\"y\":6},{\"x\":4,\"y\":5},{\"x\":4,\"y\":4}]}}";
    let next = play(game);

    assert_eq!(next, Move::Up);

}

#[test]
fn test_mange_la_pomme_2() {

    let game = "{\"game\":{\"id\":\"c896dce2-77ac-4380-9ec4-ca5869935590\"},\"turn\":43,\"board\":{\"height\":11,\"width\":11,\"food\":[{\"x\":5,\"y\":3},{\"x\":6,\"y\":3},{\"x\":9,\"y\":0}],\"snakes\":[{\"id\":\"gs_YM33Y3QDBrmjSWKhdgMwvK8C\",\"name\":\"woofers / 🐍  \u{200f}\u{200f}\u{200e} 𝙎𝙐𝙋𝙀𝙍 𝙎𝙇𝙄𝙈𝙀𝙔   \u{200f}\u{200f}\u{200e} 🐍\",\"health\":92,\"body\":[{\"x\":2,\"y\":5},{\"x\":3,\"y\":5},{\"x\":4,\"y\":5},{\"x\":4,\"y\":6},{\"x\":4,\"y\":7},{\"x\":4,\"y\":8},{\"x\":3,\"y\":8}]},{\"id\":\"gs_ccdvBRRHvcWQW8hjpS6hrYjP\",\"name\":\"xtagon / Nagini\",\"health\":75,\"body\":[{\"x\":8,\"y\":7},{\"x\":7,\"y\":7},{\"x\":7,\"y\":6},{\"x\":6,\"y\":6},{\"x\":6,\"y\":5},{\"x\":7,\"y\":5}]},{\"id\":\"gs_MRc6wbw8Hd4fGjSbghptJfx3\",\"name\":\"lduchosal / boblee-0.13\",\"health\":92,\"body\":[{\"x\":4,\"y\":3},{\"x\":4,\"y\":4},{\"x\":5,\"y\":4},{\"x\":5,\"y\":5},{\"x\":5,\"y\":6},{\"x\":5,\"y\":7}]},{\"id\":\"gs_dMrW7xTrxV6JYpDTS3XPhxhC\",\"name\":\"jhawthorn / Git Adder (experimental)\",\"health\":77,\"body\":[{\"x\":9,\"y\":4},{\"x\":10,\"y\":4},{\"x\":10,\"y\":5},{\"x\":10,\"y\":6},{\"x\":9,\"y\":6}]},{\"id\":\"gs_fwbXBmvyVwjRFwqDMYBtKdv9\",\"name\":\"jonknoll / Schneider Electric Schnake\",\"health\":99,\"body\":[{\"x\":2,\"y\":3},{\"x\":2,\"y\":4},{\"x\":3,\"y\":4},{\"x\":3,\"y\":3}]}]},\"you\":{\"id\":\"gs_MRc6wbw8Hd4fGjSbghptJfx3\",\"name\":\"lduchosal / boblee-0.13\",\"health\":92,\"body\":[{\"x\":4,\"y\":3},{\"x\":4,\"y\":4},{\"x\":5,\"y\":4},{\"x\":5,\"y\":5},{\"x\":5,\"y\":6},{\"x\":5,\"y\":7}]}}";
    let next = play(game);

    assert_eq!(next, Move::Right);

}

#[test]
fn test_mange_la_pomme_3() {

    let game = "{\"game\":{\"id\":\"11b09dfd-3e16-4106-9cf3-081a4364efae\"},\"turn\":34,\"board\":{\"height\":11,\"width\":11,\"food\":[{\"x\":0,\"y\":2},{\"x\":8,\"y\":1},{\"x\":5,\"y\":2}],\"snakes\":[{\"id\":\"gs_DSk7h6mdpxMwY7SjSGRHgYxF\",\"name\":\"duncmac16 / ProtoFeist\",\"health\":82,\"body\":[{\"x\":1,\"y\":1},{\"x\":2,\"y\":1},{\"x\":3,\"y\":1},{\"x\":4,\"y\":1},{\"x\":4,\"y\":2}]},{\"id\":\"gs_xXCmCWqdThFXPr8M9DjpYr6C\",\"name\":\"KyleDBoyd / k-snek\",\"health\":83,\"body\":[{\"x\":6,\"y\":8},{\"x\":7,\"y\":8},{\"x\":8,\"y\":8},{\"x\":9,\"y\":8},{\"x\":9,\"y\":9}]},{\"id\":\"gs_KrRQ6WqRbkDKvSHp4qFgpdyc\",\"name\":\"DevYves / snakey\",\"health\":70,\"body\":[{\"x\":3,\"y\":7},{\"x\":4,\"y\":7},{\"x\":4,\"y\":6},{\"x\":3,\"y\":6},{\"x\":2,\"y\":6}]},{\"id\":\"gs_q4h3S7jrrYGkTbg4T7ThSS9F\",\"name\":\"jonknoll / LeechySnake2018\",\"health\":92,\"body\":[{\"x\":4,\"y\":4},{\"x\":4,\"y\":5},{\"x\":5,\"y\":5},{\"x\":5,\"y\":4},{\"x\":5,\"y\":3},{\"x\":6,\"y\":3}]},{\"id\":\"gs_4Dwv3KX3GyVqvFcMY9htFXrB\",\"name\":\"lduchosal / boblee-0.13\",\"health\":66,\"body\":[{\"x\":8,\"y\":4},{\"x\":8,\"y\":5},{\"x\":7,\"y\":5}]},{\"id\":\"gs_C7bRvW4WmM7DBGMr6QhJ43VR\",\"name\":\"prplz / blockchain snake\",\"health\":92,\"body\":[{\"x\":7,\"y\":7},{\"x\":8,\"y\":7},{\"x\":9,\"y\":7},{\"x\":10,\"y\":7},{\"x\":10,\"y\":6},{\"x\":10,\"y\":5}]}]},\"you\":{\"id\":\"gs_4Dwv3KX3GyVqvFcMY9htFXrB\",\"name\":\"lduchosal / boblee-0.13\",\"health\":66,\"body\":[{\"x\":8,\"y\":4},{\"x\":8,\"y\":5},{\"x\":7,\"y\":5}]}}";
    let next = play(game);

    assert_eq!(next, Move::Up);

}

struct Error {}

impl From<std::option::NoneError> for Error {

    fn from(n: std::option::NoneError) -> Self {
        Error {}
    }
}

fn eat_my_food(game: &Game, ps: &mut Vec<Possible>) -> Result<(), Error> {


    let eat_my_food_value = 5;

    let mut nearest_my_food = Vec::new();

    for food in &game.board.food {

        let mut distances = Vec::new();

        for snake in &game.board.snakes {

            let head = &snake.body.first()?;
            let distancex = (head.x as i32 - food.x as i32).abs();
            let distancey = (head.y as i32 - food.y as i32).abs();

            let distance = distancex + distancey;

            distances.push((distance, snake));
        }

        distances.sort_by(|a,b| a.0.cmp(&b.0));
        let min = distances.first()?.0;
        distances.retain(|&x| x.0 == min);
        
        if distances.len() == 1
            && distances.first()?.1.id == game.you.id {
            nearest_my_food.push((distances.first()?.0, food));
        }

    }

    nearest_my_food.sort_by(|a,b| a.0.cmp(&b.0));
    let food = nearest_my_food.first()?.1;

    let you = game.you.body.first()?;

    let left = you.x as i32 - food.x as i32 > 0;
    let right = food.x as i32 - you.x as i32 > 0;
    let up = you.y as i32 - food.y as i32 > 0;
    let down = food.y as i32 - you.y as i32 > 0;

    for p in ps.iter_mut() {

        if p.dir == Move::Up && up {
            p.eat_my_food += 1;
            p.eat_my_food_value += eat_my_food_value;
            p.value += eat_my_food_value;
        }

        if p.dir == Move::Down && down {
            p.eat_my_food += 1;
            p.eat_my_food_value += eat_my_food_value;
            p.value += eat_my_food_value;
        }

        if p.dir == Move::Left && left {
            p.eat_my_food += 1;
            p.eat_my_food_value += eat_my_food_value;
            p.value += eat_my_food_value;
        }

        if p.dir == Move::Right && right {
            p.eat_my_food += 1;
            p.eat_my_food_value += eat_my_food_value;
            p.value += eat_my_food_value;
        }
    }

    Ok(())

}



#[test]
fn test_eat_my_snake() {

    let game ="{\"game\":{\"id\":\"3f5918e3-d5f6-4542-b63d-5de9298843be\"},\"turn\":374,\"board\":{\"height\":19,\"width\":19,\"food\":[{\"x\":2,\"y\":1},{\"x\":3,\"y\":16},{\"x\":10,\"y\":2},{\"x\":2,\"y\":12},{\"x\":16,\"y\":18},{\"x\":15,\"y\":4},{\"x\":18,\"y\":7},{\"x\":17,\"y\":2},{\"x\":1,\"y\":17},{\"x\":16,\"y\":17},{\"x\":1,\"y\":7},{\"x\":2,\"y\":16},{\"x\":13,\"y\":4},{\"x\":10,\"y\":5},{\"x\":14,\"y\":15},{\"x\":5,\"y\":17},{\"x\":18,\"y\":12},{\"x\":2,\"y\":7},{\"x\":14,\"y\":3},{\"x\":4,\"y\":1},{\"x\":7,\"y\":5}],\"snakes\":[{\"id\":\"gs_ykHTrtwyjgf7f9mKTGPgRMSM\",\"name\":\"lduchosal / kimjon-0.14\",\"health\":4,\"body\":[{\"x\":10,\"y\":6},{\"x\":11,\"y\":6},{\"x\":12,\"y\":6},{\"x\":13,\"y\":6},{\"x\":14,\"y\":6},{\"x\":14,\"y\":7},{\"x\":14,\"y\":8},{\"x\":14,\"y\":9},{\"x\":14,\"y\":10},{\"x\":14,\"y\":11},{\"x\":13,\"y\":11},{\"x\":13,\"y\":12},{\"x\":12,\"y\":12},{\"x\":12,\"y\":11},{\"x\":12,\"y\":10},{\"x\":13,\"y\":10},{\"x\":13,\"y\":9},{\"x\":12,\"y\":9},{\"x\":12,\"y\":8}]}]},\"you\":{\"id\":\"gs_ykHTrtwyjgf7f9mKTGPgRMSM\",\"name\":\"lduchosal / kimjon-0.14\",\"health\":4,\"body\":[{\"x\":10,\"y\":6},{\"x\":11,\"y\":6},{\"x\":12,\"y\":6},{\"x\":13,\"y\":6},{\"x\":14,\"y\":6},{\"x\":14,\"y\":7},{\"x\":14,\"y\":8},{\"x\":14,\"y\":9},{\"x\":14,\"y\":10},{\"x\":14,\"y\":11},{\"x\":13,\"y\":11},{\"x\":13,\"y\":12},{\"x\":12,\"y\":12},{\"x\":12,\"y\":11},{\"x\":12,\"y\":10},{\"x\":13,\"y\":10},{\"x\":13,\"y\":9},{\"x\":12,\"y\":9},{\"x\":12,\"y\":8}]}}";
    let next = play(game);

    assert_eq!(next, Move::Up);
}


fn prefer_food(game: &Game, ps: &mut Vec<Possible>) {


    for p in ps {

        let mut closest: i32 = game.board.height as i32 + game.board.width as i32;
        for f in &game.board.food {

            let distancex = (p.point.x as i32 - f.x as i32).abs();
            let distancey = (p.point.y as i32 - f.y as i32).abs();

            let distance = distancex + distancey;

            if distance < closest {
                closest = distance;
            }
        }

        p.prefer_food_distance = closest + 1; // prevent div by 0
    }

}

fn prefer_food_distance(_: &Game, ps: &mut Vec<Possible>) {

    ps.sort_by(|a, b| a.prefer_food_distance.cmp(&b.prefer_food_distance));
    let total : f32 = ps.iter().map(|item| 1.0_f32 / item.prefer_food_distance as f32).sum();

    let value = 25;
    for p in ps {
        let assigned = ((value as f32 * (1.0_f32 / p.prefer_food_distance as f32)) / total) as i32 ;
        p.value += assigned;
        p.prefer_food = assigned;
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

    let check_snakes = 100;

    for p in possibles {
        for s in &game.board.snakes {
            for b in &s.body {
                if p.point.eq(b) {
                    p.value -= check_snakes ;
                    p.check_snakes -= check_snakes;
                }
            }
        }
    }
}

fn check_tails(game: &Game, possibles: &mut Vec<Possible>) {

    let check_tails = 100;

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

            let mut value = 7;
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

fn prefer_forward_space(_: &Game, ps: &mut Vec<Possible>) {

    ps.sort_by(|a, b| a.forward_pathes_len.cmp(&b.forward_pathes_len));
    let total : i32 = ps.iter().map(|item| item.forward_pathes_len).sum();

    let value = 20;
    for p in ps {
        if total <= 0 {
            continue;
        }
        let assigned = value * p.forward_pathes_len / total;
        p.value += assigned;
        p.prefer_forward_space += assigned;
    }

}

fn append_point_me(futur: &mut Game, point: &Point) {

    for snake in &mut futur.board.snakes {
        if &snake.id == &futur.you.id {
            snake.body.insert(0, point.clone());
            futur.you = snake.clone();
        }
    }
}

fn forward_thinking(game: &Game, ps: &mut Vec<Possible>, depth: u8) {

    for p in ps {

        if p.value < 0 {
            continue;
        }

        let mut futur = game.clone();
        let mut pathes = HashSet::new();

        let root = Path { point: p.point.clone(), level: 0 };
        pathes.insert(root);

        for level in 0..depth {

            for snake in &mut futur.board.snakes {
                snake.body.pop();
            }
            futur.you.body.pop();

            let level_pathes = find_pathes(&pathes, level);
            for path in &level_pathes {
                append_point_me(&mut futur, &path.point);
            }

            for path in &level_pathes {

                let mut fps = possibles(&path.point);
                check_walls(&futur, &mut fps);
                check_snakes(&futur, &mut fps);
                check_tails(&futur, &mut fps);

                for fp in fps {
                    if fp.value > 0 {
                        let path = Path { point: fp.point.clone(), level: level +1 };
                        pathes.insert(path);
                    }
                }

            }

        }

        let forward_thinking =  30  ; // 15
        p.forward_pathes_len = pathes.len() as i32;

        if pathes.len() < 4 * depth as usize {
            p.value -= forward_thinking;
            p.forward_thinking -= forward_thinking;
            p.forward_pathes = pathes;
        }
    }
}

#[test]
fn test_forward_think_better() {

    let game = "{\"game\":{\"id\":\"7ab8e619-c7c3-47fa-9eb2-4cb3470a2dfa\"},\"turn\":533,\"board\":{\"height\":11,\"width\":11,\"food\":[{\"x\":8,\"y\":7},{\"x\":9,\"y\":4}],\"snakes\":[{\"id\":\"gs_VqGdRdgP8Bc8t3rrYF4PhHB6\",\"name\":\"lduchosal / dev\",\"health\":85,\"body\":[{\"x\":5,\"y\":4},{\"x\":5,\"y\":5},{\"x\":4,\"y\":5},{\"x\":3,\"y\":5},{\"x\":2,\"y\":5},{\"x\":1,\"y\":5},{\"x\":1,\"y\":6},{\"x\":0,\"y\":6},{\"x\":0,\"y\":7},{\"x\":1,\"y\":7},{\"x\":2,\"y\":7},{\"x\":2,\"y\":6},{\"x\":3,\"y\":6},{\"x\":3,\"y\":7},{\"x\":3,\"y\":8},{\"x\":2,\"y\":8},{\"x\":2,\"y\":9},{\"x\":3,\"y\":9},{\"x\":4,\"y\":9},{\"x\":4,\"y\":10},{\"x\":5,\"y\":10},{\"x\":6,\"y\":10},{\"x\":7,\"y\":10},{\"x\":7,\"y\":9},{\"x\":8,\"y\":9},{\"x\":8,\"y\":8},{\"x\":7,\"y\":8},{\"x\":6,\"y\":8},{\"x\":6,\"y\":9},{\"x\":5,\"y\":9},{\"x\":5,\"y\":8},{\"x\":4,\"y\":8},{\"x\":4,\"y\":7},{\"x\":4,\"y\":6},{\"x\":5,\"y\":6},{\"x\":6,\"y\":6},{\"x\":6,\"y\":5},{\"x\":7,\"y\":5},{\"x\":7,\"y\":4},{\"x\":7,\"y\":3},{\"x\":6,\"y\":3},{\"x\":6,\"y\":2},{\"x\":5,\"y\":2},{\"x\":5,\"y\":1},{\"x\":5,\"y\":0},{\"x\":4,\"y\":0},{\"x\":3,\"y\":0},{\"x\":2,\"y\":0},{\"x\":1,\"y\":0},{\"x\":1,\"y\":1},{\"x\":0,\"y\":1},{\"x\":0,\"y\":2},{\"x\":0,\"y\":3}]}]},\"you\":{\"id\":\"gs_VqGdRdgP8Bc8t3rrYF4PhHB6\",\"name\":\"lduchosal / dev\",\"health\":85,\"body\":[{\"x\":5,\"y\":4},{\"x\":5,\"y\":5},{\"x\":4,\"y\":5},{\"x\":3,\"y\":5},{\"x\":2,\"y\":5},{\"x\":1,\"y\":5},{\"x\":1,\"y\":6},{\"x\":0,\"y\":6},{\"x\":0,\"y\":7},{\"x\":1,\"y\":7},{\"x\":2,\"y\":7},{\"x\":2,\"y\":6},{\"x\":3,\"y\":6},{\"x\":3,\"y\":7},{\"x\":3,\"y\":8},{\"x\":2,\"y\":8},{\"x\":2,\"y\":9},{\"x\":3,\"y\":9},{\"x\":4,\"y\":9},{\"x\":4,\"y\":10},{\"x\":5,\"y\":10},{\"x\":6,\"y\":10},{\"x\":7,\"y\":10},{\"x\":7,\"y\":9},{\"x\":8,\"y\":9},{\"x\":8,\"y\":8},{\"x\":7,\"y\":8},{\"x\":6,\"y\":8},{\"x\":6,\"y\":9},{\"x\":5,\"y\":9},{\"x\":5,\"y\":8},{\"x\":4,\"y\":8},{\"x\":4,\"y\":7},{\"x\":4,\"y\":6},{\"x\":5,\"y\":6},{\"x\":6,\"y\":6},{\"x\":6,\"y\":5},{\"x\":7,\"y\":5},{\"x\":7,\"y\":4},{\"x\":7,\"y\":3},{\"x\":6,\"y\":3},{\"x\":6,\"y\":2},{\"x\":5,\"y\":2},{\"x\":5,\"y\":1},{\"x\":5,\"y\":0},{\"x\":4,\"y\":0},{\"x\":3,\"y\":0},{\"x\":2,\"y\":0},{\"x\":1,\"y\":0},{\"x\":1,\"y\":1},{\"x\":0,\"y\":1},{\"x\":0,\"y\":2},{\"x\":0,\"y\":3}]}}";
    let next = play(game);

    assert_eq!(next, Move::Up);
}

fn find_pathes(pathes: &HashSet<Path>, level: u8) -> HashSet<Path> {
    let mut result = HashSet::<Path>::new();

    for path in pathes {
        if path.level == level {
            let chosen = path.clone();
            result.insert(chosen);
        }
    }

    result
}

fn hunt_snakes(game: &Game, ps: &mut Vec<Possible>) {

    let mut best_snake = 0;
    let mut best_second_snake = 0;

    for snake in &game.board.snakes {

        let body_len = snake.body.len();
        if body_len > best_snake {
            best_snake = body_len;
        }
        else {
            if body_len > best_second_snake {
                best_second_snake = body_len;
            }
        }
    }

    if best_snake >= best_second_snake + 3 
        && best_snake == game.you.body.len()
    {
        println!("best snake.. go for a fight");
    }
}

fn enroule_ton_snake(game: &Game, ps: &mut Vec<Possible>) {


}

#[test]
fn test_enroule_ton_snake() {

    let game = "
    {\"game\":
        {\"id\":\"d770c179-7f52-45e3-bf85-288345f7a359\"},
        \"turn\":257,
        \"board\":{
            \"height\":11,
            \"width\":11,
            \"food\":[{\"x\":4,\"y\":0},{\"x\":1,\"y\":8}],
            \"snakes\":[
                {\"id\":\"gs_dQbRY6wwBmHx6cp994SKqrp6\",
                    \"name\":\"lduchosal / kimjon-0.14\",
                    \"health\":99,
                    \"body\":[{\"x\":7,\"y\":0},{\"x\":6,\"y\":0},{\"x\":5,\"y\":0},
                        {\"x\":5,\"y\":1},{\"x\":5,\"y\":2},{\"x\":5,\"y\":3},{\"x\":5,\"y\":4},
                        {\"x\":6,\"y\":4},
                        {\"x\":7,\"y\":4},
                        {\"x\":8,\"y\":4},
                        {\"x\":9,\"y\":4},{\"x\":9,\"y\":5},
                        {\"x\":10,\"y\":5},{\"x\":10,\"y\":6},{\"x\":10,\"y\":7},{\"x\":10,\"y\":8},
                        {\"x\":9,\"y\":8},
                        {\"x\":8,\"y\":8},
                        {\"x\":7,\"y\":8},
                        {\"x\":7,\"y\":9}]},

                {\"id\":\"gs_wrrQW4TFXPdQfrDCFfh6vfDM\",
                    \"name\":\"zijian-chen96 / pinkpinkpig\",
                    \"health\":29,
                    \"body\":[
                        {\"x\":2,\"y\":3},{\"x\":2,\"y\":2},
                        {\"x\":3,\"y\":2},{\"x\":3,\"y\":1},
                        {\"x\":4,\"y\":1},{\"x\":4,\"y\":2},{\"x\":4,\"y\":3},{\"x\":4,\"y\":4},{\"x\":4,\"y\":5},{\"x\":4,\"y\":6},{\"x\":4,\"y\":7},{\"x\":4,\"y\":8},
                        {\"x\":3,\"y\":8}]}
                ]},
            \"you\":{
                \"id\":\"gs_dQbRY6wwBmHx6cp994SKqrp6\",
                \"name\":\"lduchosal / kimjon-0.14\",
                \"health\":99,
                \"body\":[{\"x\":7,\"y\":0},{\"x\":6,\"y\":0},{\"x\":5,\"y\":0},{\"x\":5,\"y\":1},{\"x\":5,\"y\":2},{\"x\":5,\"y\":3},{\"x\":5,\"y\":4},{\"x\":6,\"y\":4},{\"x\":7,\"y\":4},{\"x\":8,\"y\":4},{\"x\":9,\"y\":4},{\"x\":9,\"y\":5},{\"x\":10,\"y\":5},{\"x\":10,\"y\":6},{\"x\":10,\"y\":7},{\"x\":10,\"y\":8},{\"x\":9,\"y\":8},{\"x\":8,\"y\":8},{\"x\":7,\"y\":8},{\"x\":7,\"y\":9}]}}";

    play(game);
}

fn build_futur<'t>(game: &Game) -> Arena<Point> {

    let head = &game.you.body[0];
    let mut a = Arena::new();
    let arena = &mut a;
    let root = arena.new_node(head.clone());
    build_futur_nodes(game, arena, root, 0);

    a
}

fn print_pathes(pathes: &Vec<Vec<Point>>) {

    let max = pathes.last().unwrap().len();
    let min = pathes.first().unwrap().len();
    let count = pathes.len();
    println!("min: {}", min);
    println!("max: {}", max);
    println!("count: {}", count);

    for path in pathes {

        //if !path.contains(&Point { x: 10, y: 4 }) {
        if path.len() < max 
            && path.len() > min {
            continue;
        }

        print!("{} [ ", path.len());
        for point in path {
            print!("{} ", point);
        } 
        println!("]");
    }
}

fn convert_pathes(arena: &mut Arena<Point>) -> Vec<Vec<Point>> {

    let mut leaves = Vec::new();
    for node in arena.iter() {
        //println!("Node:Deserialize {} {:?} {:?}", node.data, node.first_child(), node.last_child());
        match node.last_child() {
            None => leaves.push(node),
            Some(_) => {}
        }
    }

    let mut pathes = Vec::new();
    for leaf in leaves {

        match leaf.parent() {
            None => {},
            Some(parent_id) => {

                let mut path = get_parent_data(arena, parent_id);
                path.reverse();
                path.push(leaf.data.clone());
                pathes.push(path);
            }
        }
    }

    pathes
}

fn build_futur_nodes<'t>(game: &Game, arena: &mut Arena<Point>, parent: NodeId, level: u8) {

    if arena.count() > 4000 { // Too deep is the ocean the ocean
        return;
    }

    let data = get_data(arena, parent);
    for (i, j) in next_moves() {

        let next = Point {
            x: &data.x + i,
            y: &data.y + j
        };

        if outside_board(&game.board, &next) {
            continue;
        }

        if parent_already_visited(arena, parent, &next) {
            continue;
        }

        if snake_present(&game.board, &next) {
            continue;
        }

        let next_node = arena.new_node(next);
        parent.append(next_node, arena);

        build_futur_nodes(game, arena, next_node, level + 1);
    }
}

fn next_moves() -> Vec<(i16, i16)> {
    vec![(0, 1), (1, 0), (0, -1), (-1, 0)]
}

fn snake_present(board: &Board, point: &Point) -> bool {
    for snake in &board.snakes {
        for body in &snake.body {
            if body.eq(point) {
                return true;
            }
        }
    }
    false
}

fn outside_board(board: &Board, point: &Point) -> bool {
    point.x < 0 
    || point.y < 0
    || point.x >= board.width as i16
    || point.y >= board.height as i16
}

fn parent_already_visited(arena: &mut Arena<Point>, node: NodeId, point: &Point) -> bool {

    let datas = get_parent_data(arena, node);
    for data in datas {
        if data == *point {
            return true;
        }
    }
    return false;
}

fn get_parent_data(arena: &Arena<Point>, node: NodeId) -> Vec::<Point> {
    let mut result = Vec::<Point>::new();

    let parents = node.ancestors(arena);

    for parent in parents.into_iter() {
        let data = arena.get(parent).unwrap().data.clone();
        result.push(data);
    }

    result
}

fn get_data(arena: &mut Arena<Point>, node_id: NodeId) -> Point {
    let node = arena.get(node_id).unwrap();
    node.data.clone()
}

// pub struct Node {
//     data: Point,
//     children: Vec<Node>,
//     parents: Vec<Node>,
// }

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
    prefer_food_distance: i32,
    eat_my_food_value: i32,
    eat_my_food: i32,
    hit_or_leave: i32,
    look_for_tail: i32,
    forward_thinking: i32,
    forward_pathes: HashSet<Path>,
    forward_pathes_len: i32,
    prefer_forward_space: i32,

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
            prefer_food_distance: 0,
            eat_my_food_value: 0,
            eat_my_food: 0,
            hit_or_leave: 0,
            look_for_tail: 0,
            forward_thinking: 0,
            kill_heads: 0,
            forward_pathes: HashSet::new(),
            forward_pathes_len: 0,
            prefer_forward_space: 0,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Path {
    point: Point,
    level: u8,
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.point.x, self.point.y, self.level)
    }
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq

)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Eq, Hash)]
pub struct Point {
    #[serde(rename = "x")]
    pub x: i16,

    #[serde(rename = "y")]
    pub y: i16,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.x, self.y)
    }
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


#[test]
fn test_panic() {

    let game = "{\"game\":{\"id\":\"3a7a940f-6790-457f-aaf2-bc6711915d24\"},\"turn\":520,\"board\":{\"height\":11,\"width\":11,\"food\":[{\"x\":10,\"y\":10},{\"x\":1,\"y\":0},{\"x\":6,\"y\":10},{\"x\":9,\"y\":4},{\"x\":8,\"y\":2},{\"x\":5,\"y\":8}],\"snakes\":[{\"id\":\"gs_JKthCR7JhDktSWp3gyGVhQGX\",\"name\":\"lduchosal / albertfish-dev\",\"health\":87,\"body\":[{\"x\":1,\"y\":5},{\"x\":2,\"y\":5},{\"x\":2,\"y\":4},{\"x\":2,\"y\":3},{\"x\":1,\"y\":3},{\"x\":1,\"y\":4},{\"x\":0,\"y\":4},{\"x\":0,\"y\":5},{\"x\":0,\"y\":6},{\"x\":1,\"y\":6},{\"x\":2,\"y\":6},{\"x\":2,\"y\":7},{\"x\":2,\"y\":8},{\"x\":1,\"y\":8},{\"x\":0,\"y\":8},{\"x\":0,\"y\":9},{\"x\":1,\"y\":9},{\"x\":2,\"y\":9},{\"x\":3,\"y\":9},{\"x\":3,\"y\":8},{\"x\":3,\"y\":7},{\"x\":4,\"y\":7},{\"x\":4,\"y\":6},{\"x\":3,\"y\":6},{\"x\":3,\"y\":5},{\"x\":3,\"y\":4},{\"x\":3,\"y\":3},{\"x\":3,\"y\":2},{\"x\":2,\"y\":2},{\"x\":1,\"y\":2},{\"x\":0,\"y\":2},{\"x\":0,\"y\":1},{\"x\":1,\"y\":1},{\"x\":2,\"y\":1},{\"x\":3,\"y\":1},{\"x\":4,\"y\":1},{\"x\":4,\"y\":2},{\"x\":5,\"y\":2},{\"x\":5,\"y\":3},{\"x\":4,\"y\":3},{\"x\":4,\"y\":4},{\"x\":5,\"y\":4},{\"x\":6,\"y\":4},{\"x\":7,\"y\":4},{\"x\":7,\"y\":5},{\"x\":8,\"y\":5},{\"x\":9,\"y\":5},{\"x\":10,\"y\":5}]}]},\"you\":{\"id\":\"gs_JKthCR7JhDktSWp3gyGVhQGX\",\"name\":\"lduchosal / albertfish-dev\",\"health\":87,\"body\":[{\"x\":1,\"y\":5},{\"x\":2,\"y\":5},{\"x\":2,\"y\":4},{\"x\":2,\"y\":3},{\"x\":1,\"y\":3},{\"x\":1,\"y\":4},{\"x\":0,\"y\":4},{\"x\":0,\"y\":5},{\"x\":0,\"y\":6},{\"x\":1,\"y\":6},{\"x\":2,\"y\":6},{\"x\":2,\"y\":7},{\"x\":2,\"y\":8},{\"x\":1,\"y\":8},{\"x\":0,\"y\":8},{\"x\":0,\"y\":9},{\"x\":1,\"y\":9},{\"x\":2,\"y\":9},{\"x\":3,\"y\":9},{\"x\":3,\"y\":8},{\"x\":3,\"y\":7},{\"x\":4,\"y\":7},{\"x\":4,\"y\":6},{\"x\":3,\"y\":6},{\"x\":3,\"y\":5},{\"x\":3,\"y\":4},{\"x\":3,\"y\":3},{\"x\":3,\"y\":2},{\"x\":2,\"y\":2},{\"x\":1,\"y\":2},{\"x\":0,\"y\":2},{\"x\":0,\"y\":1},{\"x\":1,\"y\":1},{\"x\":2,\"y\":1},{\"x\":3,\"y\":1},{\"x\":4,\"y\":1},{\"x\":4,\"y\":2},{\"x\":5,\"y\":2},{\"x\":5,\"y\":3},{\"x\":4,\"y\":3},{\"x\":4,\"y\":4},{\"x\":5,\"y\":4},{\"x\":6,\"y\":4},{\"x\":7,\"y\":4},{\"x\":7,\"y\":5},{\"x\":8,\"y\":5},{\"x\":9,\"y\":5},{\"x\":10,\"y\":5}]}}";
    play(game);

}

#[test]
fn test_food() {

    let game = "{\"game\":{\"id\":\"55cbf87a-3bf5-4137-b985-4d3c38ebe69b\"},\"turn\":15,\"board\":{\"height\":11,\"width\":11,\"food\":[{\"x\":2,\"y\":4},{\"x\":3,\"y\":9}],\"snakes\":[{\"id\":\"gs_CmFXRMdmvxkkjYm6QtDxVYbG\",\"name\":\"s4-ricky / Turing Snake\",\"health\":100,\"body\":[{\"x\":5,\"y\":2},{\"x\":5,\"y\":1},{\"x\":5,\"y\":0},{\"x\":4,\"y\":0},{\"x\":4,\"y\":1},{\"x\":4,\"y\":1}]},{\"id\":\"gs_gq3RmBrWXfjGqy9yyGCyqkMR\",\"name\":\"chunthebear / Behir\",\"health\":100,\"body\":[{\"x\":6,\"y\":9},{\"x\":7,\"y\":9},{\"x\":8,\"y\":9},{\"x\":9,\"y\":9},{\"x\":9,\"y\":8},{\"x\":9,\"y\":8}]},{\"id\":\"gs_bHSkfffHVxqCfW8XHtdvfTjT\",\"name\":\"tagg7 / SNAKEFACE 2.0\",\"health\":98,\"body\":[{\"x\":5,\"y\":10},{\"x\":6,\"y\":10},{\"x\":7,\"y\":10},{\"x\":8,\"y\":10}]},{\"id\":\"gs_hV4fwPtH6RCCkDdqgfSFWTgK\",\"name\":\"lduchosal / albertfish-dev\",\"health\":85,\"body\":[{\"x\":3,\"y\":4},{\"x\":3,\"y\":5},{\"x\":4,\"y\":5}]}]},\"you\":{\"id\":\"gs_hV4fwPtH6RCCkDdqgfSFWTgK\",\"name\":\"lduchosal / albertfish-dev\",\"health\":85,\"body\":[{\"x\":3,\"y\":4},{\"x\":3,\"y\":5},{\"x\":4,\"y\":5}]}}";
    play(game);

}


#[test]
fn test_food_bug() {

    let game = "{\"game\":{\"id\":\"d395766e-f411-4800-8184-0be8aa35d6ae\"},\"turn\":432,\"board\":{\"height\":19,\"width\":19,\"food\":[{\"x\":3,\"y\":13},{\"x\":1,\"y\":9},{\"x\":1,\"y\":15},{\"x\":1,\"y\":14}],\"snakes\":[{\"id\":\"gs_BdRKFSDS8SMFkfTpSxhTGpCF\",\"name\":\"lduchosal / albertfish-dev\",\"health\":90,\"body\":[{\"x\":9,\"y\":11},{\"x\":8,\"y\":11},{\"x\":8,\"y\":12},{\"x\":8,\"y\":13},{\"x\":9,\"y\":13},{\"x\":10,\"y\":13},{\"x\":10,\"y\":12},{\"x\":10,\"y\":11},{\"x\":11,\"y\":11},{\"x\":12,\"y\":11},{\"x\":13,\"y\":11},{\"x\":13,\"y\":12},{\"x\":12,\"y\":12},{\"x\":12,\"y\":13},{\"x\":11,\"y\":13},{\"x\":11,\"y\":14},{\"x\":10,\"y\":14},{\"x\":10,\"y\":15},{\"x\":9,\"y\":15},{\"x\":8,\"y\":15},{\"x\":8,\"y\":16},{\"x\":8,\"y\":17},{\"x\":7,\"y\":17},{\"x\":7,\"y\":16},{\"x\":7,\"y\":15},{\"x\":7,\"y\":14},{\"x\":7,\"y\":13},{\"x\":7,\"y\":12},{\"x\":7,\"y\":11},{\"x\":7,\"y\":10},{\"x\":8,\"y\":10},{\"x\":8,\"y\":9},{\"x\":8,\"y\":8},{\"x\":8,\"y\":7},{\"x\":9,\"y\":7},{\"x\":10,\"y\":7},{\"x\":11,\"y\":7},{\"x\":11,\"y\":6},{\"x\":11,\"y\":5},{\"x\":11,\"y\":4},{\"x\":12,\"y\":4}]}]},\"you\":{\"id\":\"gs_BdRKFSDS8SMFkfTpSxhTGpCF\",\"name\":\"lduchosal / albertfish-dev\",\"health\":90,\"body\":[{\"x\":9,\"y\":11},{\"x\":8,\"y\":11},{\"x\":8,\"y\":12},{\"x\":8,\"y\":13},{\"x\":9,\"y\":13},{\"x\":10,\"y\":13},{\"x\":10,\"y\":12},{\"x\":10,\"y\":11},{\"x\":11,\"y\":11},{\"x\":12,\"y\":11},{\"x\":13,\"y\":11},{\"x\":13,\"y\":12},{\"x\":12,\"y\":12},{\"x\":12,\"y\":13},{\"x\":11,\"y\":13},{\"x\":11,\"y\":14},{\"x\":10,\"y\":14},{\"x\":10,\"y\":15},{\"x\":9,\"y\":15},{\"x\":8,\"y\":15},{\"x\":8,\"y\":16},{\"x\":8,\"y\":17},{\"x\":7,\"y\":17},{\"x\":7,\"y\":16},{\"x\":7,\"y\":15},{\"x\":7,\"y\":14},{\"x\":7,\"y\":13},{\"x\":7,\"y\":12},{\"x\":7,\"y\":11},{\"x\":7,\"y\":10},{\"x\":8,\"y\":10},{\"x\":8,\"y\":9},{\"x\":8,\"y\":8},{\"x\":8,\"y\":7},{\"x\":9,\"y\":7},{\"x\":10,\"y\":7},{\"x\":11,\"y\":7},{\"x\":11,\"y\":6},{\"x\":11,\"y\":5},{\"x\":11,\"y\":4},{\"x\":12,\"y\":4}]}}";
    let next = play(game);

    assert_eq!(next, Move::Up);

}

#[test]
fn test_food_distance() {

    let game = "{\"game\":{\"id\":\"e2e2f508-e1a4-452c-b33b-b053b3f8b217\"},\"turn\":0,\"board\":{\"height\":11,\"width\":11,\"food\":[{\"x\":10,\"y\":4}],\"snakes\":[{\"id\":\"gs_VJb3DHBtw8fdBVMv4KTFG3YV\",\"name\":\"lduchosal / albertfish-dev\",\"health\":100,\"body\":[{\"x\":1,\"y\":1},{\"x\":1,\"y\":1},{\"x\":1,\"y\":1}]}]},\"you\":{\"id\":\"gs_VJb3DHBtw8fdBVMv4KTFG3YV\",\"name\":\"lduchosal / albertfish-dev\",\"health\":100,\"body\":[{\"x\":1,\"y\":1},{\"x\":1,\"y\":1},{\"x\":1,\"y\":1}]}}";
    let next = play(game);

    assert_eq!(next, Move::Right);

}


#[test]
fn test_too_hungry() {

    let game = "{\"game\":{\"id\":\"068a60c3-41f9-4801-bb1b-0b29bcad580c\"},\"turn\":62,\"board\":{\"height\":11,\"width\":11,\"food\":[{\"x\":7,\"y\":10},{\"x\":2,\"y\":7}],\"snakes\":[{\"id\":\"gs_MY3MPw9d7YMwddqmpxMCk3BY\",\"name\":\"tmastrom / tommy_yum\",\"health\":98,\"body\":[{\"x\":10,\"y\":4},{\"x\":10,\"y\":3},{\"x\":10,\"y\":2},{\"x\":10,\"y\":1},{\"x\":9,\"y\":1},{\"x\":9,\"y\":0},{\"x\":8,\"y\":0},{\"x\":7,\"y\":0},{\"x\":6,\"y\":0}]},{\"id\":\"gs_Px7dBQ88xqmXHMjKqDxftWm9\",\"name\":\"CyrusSA / Jah-Snake\",\"health\":100,\"body\":[{\"x\":8,\"y\":10},{\"x\":8,\"y\":9},{\"x\":8,\"y\":8},{\"x\":9,\"y\":8},{\"x\":9,\"y\":8}]},{\"id\":\"gs_d8qRXT3JtWf3HywSVcXMMXb7\",\"name\":\"CptMayday / One Snek - Two Snek\",\"health\":94,\"body\":[{\"x\":3,\"y\":3},{\"x\":3,\"y\":4},{\"x\":3,\"y\":5},{\"x\":4,\"y\":5},{\"x\":5,\"y\":5}]},{\"id\":\"gs_cJP96brgg6PYx8gC6KxWjbqT\",\"name\":\"lduchosal / robknox-0.12\",\"health\":100,\"body\":[{\"x\":7,\"y\":9},{\"x\":6,\"y\":9},{\"x\":5,\"y\":9},{\"x\":5,\"y\":10},{\"x\":4,\"y\":10},{\"x\":3,\"y\":10},{\"x\":2,\"y\":10},{\"x\":2,\"y\":10}]}]},\"you\":{\"id\":\"gs_cJP96brgg6PYx8gC6KxWjbqT\",\"name\":\"lduchosal / robknox-0.12\",\"health\":100,\"body\":[{\"x\":7,\"y\":9},{\"x\":6,\"y\":9},{\"x\":5,\"y\":9},{\"x\":5,\"y\":10},{\"x\":4,\"y\":10},{\"x\":3,\"y\":10},{\"x\":2,\"y\":10},{\"x\":2,\"y\":10}]}}";
    let next = play(game);

    assert_eq!(next, Move::Up);


}

#[test]
fn test_too_keep_on_following_tail() {

    let game = "{\"game\":{\"id\":\"ea6e9712-b306-4526-a34d-180e1beaa42a\"},\"turn\":907,\"board\":{\"height\":11,\"width\":11,\"food\":[{\"x\":7,\"y\":9},{\"x\":7,\"y\":2},{\"x\":9,\"y\":2}],\"snakes\":[{\"id\":\"gs_hCwhtHpy8RTkrJh7M464BX8P\",\"name\":\"lduchosal / robknox-dev\",\"health\":99,\"body\":[{\"x\":10,\"y\":9},{\"x\":9,\"y\":9},{\"x\":9,\"y\":8},{\"x\":9,\"y\":7},{\"x\":9,\"y\":6},{\"x\":10,\"y\":6},{\"x\":10,\"y\":5},{\"x\":9,\"y\":5},{\"x\":8,\"y\":5},{\"x\":8,\"y\":4},{\"x\":8,\"y\":3},{\"x\":9,\"y\":3},{\"x\":9,\"y\":4},{\"x\":10,\"y\":4},{\"x\":10,\"y\":3},{\"x\":10,\"y\":2},{\"x\":10,\"y\":1},{\"x\":10,\"y\":0},{\"x\":9,\"y\":0},{\"x\":8,\"y\":0},{\"x\":7,\"y\":0},{\"x\":7,\"y\":1},{\"x\":6,\"y\":1},{\"x\":5,\"y\":1},{\"x\":4,\"y\":1},{\"x\":4,\"y\":0},{\"x\":3,\"y\":0},{\"x\":2,\"y\":0},{\"x\":2,\"y\":1},{\"x\":2,\"y\":2},{\"x\":2,\"y\":3},{\"x\":3,\"y\":3},{\"x\":3,\"y\":4},{\"x\":3,\"y\":5},{\"x\":4,\"y\":5},{\"x\":4,\"y\":6},{\"x\":3,\"y\":6},{\"x\":3,\"y\":7},{\"x\":2,\"y\":7},{\"x\":2,\"y\":6},{\"x\":2,\"y\":5},{\"x\":2,\"y\":4},{\"x\":1,\"y\":4},{\"x\":1,\"y\":3},{\"x\":1,\"y\":2},{\"x\":1,\"y\":1},{\"x\":1,\"y\":0},{\"x\":0,\"y\":0},{\"x\":0,\"y\":1},{\"x\":0,\"y\":2},{\"x\":0,\"y\":3},{\"x\":0,\"y\":4},{\"x\":0,\"y\":5},{\"x\":0,\"y\":6},{\"x\":1,\"y\":6},{\"x\":1,\"y\":7},{\"x\":1,\"y\":8},{\"x\":2,\"y\":8},{\"x\":3,\"y\":8},{\"x\":4,\"y\":8},{\"x\":4,\"y\":9},{\"x\":3,\"y\":9},{\"x\":3,\"y\":10},{\"x\":4,\"y\":10},{\"x\":5,\"y\":10},{\"x\":5,\"y\":9},{\"x\":5,\"y\":8},{\"x\":5,\"y\":7},{\"x\":6,\"y\":7},{\"x\":6,\"y\":6},{\"x\":6,\"y\":5},{\"x\":6,\"y\":4},{\"x\":5,\"y\":4},{\"x\":5,\"y\":3},{\"x\":6,\"y\":3},{\"x\":7,\"y\":3},{\"x\":7,\"y\":4},{\"x\":7,\"y\":5},{\"x\":7,\"y\":6},{\"x\":7,\"y\":7},{\"x\":8,\"y\":7},{\"x\":8,\"y\":8},{\"x\":8,\"y\":9},{\"x\":8,\"y\":10},{\"x\":9,\"y\":10},{\"x\":10,\"y\":10}]}]},\"you\":{\"id\":\"gs_hCwhtHpy8RTkrJh7M464BX8P\",\"name\":\"lduchosal / robknox-dev\",\"health\":99,\"body\":[{\"x\":10,\"y\":9},{\"x\":9,\"y\":9},{\"x\":9,\"y\":8},{\"x\":9,\"y\":7},{\"x\":9,\"y\":6},{\"x\":10,\"y\":6},{\"x\":10,\"y\":5},{\"x\":9,\"y\":5},{\"x\":8,\"y\":5},{\"x\":8,\"y\":4},{\"x\":8,\"y\":3},{\"x\":9,\"y\":3},{\"x\":9,\"y\":4},{\"x\":10,\"y\":4},{\"x\":10,\"y\":3},{\"x\":10,\"y\":2},{\"x\":10,\"y\":1},{\"x\":10,\"y\":0},{\"x\":9,\"y\":0},{\"x\":8,\"y\":0},{\"x\":7,\"y\":0},{\"x\":7,\"y\":1},{\"x\":6,\"y\":1},{\"x\":5,\"y\":1},{\"x\":4,\"y\":1},{\"x\":4,\"y\":0},{\"x\":3,\"y\":0},{\"x\":2,\"y\":0},{\"x\":2,\"y\":1},{\"x\":2,\"y\":2},{\"x\":2,\"y\":3},{\"x\":3,\"y\":3},{\"x\":3,\"y\":4},{\"x\":3,\"y\":5},{\"x\":4,\"y\":5},{\"x\":4,\"y\":6},{\"x\":3,\"y\":6},{\"x\":3,\"y\":7},{\"x\":2,\"y\":7},{\"x\":2,\"y\":6},{\"x\":2,\"y\":5},{\"x\":2,\"y\":4},{\"x\":1,\"y\":4},{\"x\":1,\"y\":3},{\"x\":1,\"y\":2},{\"x\":1,\"y\":1},{\"x\":1,\"y\":0},{\"x\":0,\"y\":0},{\"x\":0,\"y\":1},{\"x\":0,\"y\":2},{\"x\":0,\"y\":3},{\"x\":0,\"y\":4},{\"x\":0,\"y\":5},{\"x\":0,\"y\":6},{\"x\":1,\"y\":6},{\"x\":1,\"y\":7},{\"x\":1,\"y\":8},{\"x\":2,\"y\":8},{\"x\":3,\"y\":8},{\"x\":4,\"y\":8},{\"x\":4,\"y\":9},{\"x\":3,\"y\":9},{\"x\":3,\"y\":10},{\"x\":4,\"y\":10},{\"x\":5,\"y\":10},{\"x\":5,\"y\":9},{\"x\":5,\"y\":8},{\"x\":5,\"y\":7},{\"x\":6,\"y\":7},{\"x\":6,\"y\":6},{\"x\":6,\"y\":5},{\"x\":6,\"y\":4},{\"x\":5,\"y\":4},{\"x\":5,\"y\":3},{\"x\":6,\"y\":3},{\"x\":7,\"y\":3},{\"x\":7,\"y\":4},{\"x\":7,\"y\":5},{\"x\":7,\"y\":6},{\"x\":7,\"y\":7},{\"x\":8,\"y\":7},{\"x\":8,\"y\":8},{\"x\":8,\"y\":9},{\"x\":8,\"y\":10},{\"x\":9,\"y\":10},{\"x\":10,\"y\":10}]}}";
    let next = play(game);

    assert_eq!(next, Move::Down);

}

