use rand::Rng;
use std::{sync::mpsc, thread, time};
use tty_read::{ReaderOptions, TermReader};

struct GameField {
    width: usize,
    height: usize,
    tiles: Vec<Vec<char>>,
    wall_tile: char,
    float_tile: char,
}

struct Snake {
    skin: char,
    lengh: usize,
    body: Vec<SnakeBody>,
    direction: Direction,
    field_width: usize,
    field_height: usize,
}

#[derive(PartialEq)]
struct SnakeBody {
    x: usize,
    y: usize,
}

struct Fruit {
    x: usize,
    y: usize,
    field_width: usize,
    field_height: usize,
    skin: char,
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl GameField {
    fn new(width: usize, height: usize, wall_tile: char, float_tile: char) -> Self {
        let tiles = Vec::<Vec<char>>::new();
        return GameField {
            width,
            height,
            wall_tile,
            float_tile,
            tiles,
        };
    }
    fn init(&mut self) {
        for y in 0..self.height {
            self.tiles.push(Vec::<char>::new());
            for _ in 0..self.width {
                self.tiles.get_mut(y).unwrap().push(' ');
            }
        }
    }
    fn tile_reset(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = if y == 0 || y == self.height - 1 || x == 0 || x == self.width - 1 {
                    self.wall_tile
                } else {
                    self.float_tile
                };
                self.tile_update(x, y, tile);
            }
        }
    }
    fn tile_update(&mut self, x: usize, y: usize, tile: char) {
        let line = self.tiles.get_mut(y).unwrap();
        line[x] = tile;
    }
    fn render(&self) {
        print!("\x1B[2J\x1B[H");
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = self.tiles.get(y).unwrap().get(x).unwrap();
                print!("{}", tile);
            }
            //print!("\x1B[1E");
            print!("\r\n");
        }
    }
}

impl Snake {
    fn new(skin: char, lengh: usize, field_width: usize, field_height: usize) -> Self {
        let body = Vec::<SnakeBody>::new();
        let direction = Direction::Up;
        return Snake {
            skin,
            lengh,
            body,
            direction,
            field_width,
            field_height,
        };
    }
    fn init(&mut self, default_x: usize, default_y: usize) {
        for i in 0..self.lengh {
            let body = SnakeBody {
                x: default_x + i,
                y: default_y,
            };
            self.body.push(body);
        }
    }
    fn get_body(&self, index: usize) -> Option<&SnakeBody> {
        return self.body.get(index);
    }
    fn get_lengh(&self) -> usize {
        return self.lengh;
    }
    fn set_direction(&mut self, direction: Direction) {
        if self.direction == Direction::Up && direction == Direction::Down {
        } else if self.direction == Direction::Down && direction == Direction::Up {
        } else if self.direction == Direction::Left && direction == Direction::Right {
        } else if self.direction == Direction::Right && direction == Direction::Left {
        } else {
            self.direction = direction;
        }
    }
    fn move_body(&mut self) -> Result<(), String> {
        self.body.pop().unwrap();
        let head = self.body.first().unwrap();
        let new_head = match self.direction {
            Direction::Up => SnakeBody {
                x: head.x,
                y: head.y - 1,
            },
            Direction::Down => SnakeBody {
                x: head.x,
                y: head.y + 1,
            },
            Direction::Left => SnakeBody {
                x: head.x - 1,
                y: head.y,
            },
            Direction::Right => SnakeBody {
                x: head.x + 1,
                y: head.y,
            },
        };
        if new_head.x == 0
            || new_head.x == self.field_width - 1
            || new_head.y == 0
            || new_head.y == self.field_height - 1
        {
            return Err("field out side".to_string());
        }
        for i in 0..self.lengh - 1 {
            for j in 0..self.lengh - 1 {
                if i == j {
                    continue;
                }
                let body = self.body.get(i).unwrap();
                let other_body = self.body.get(j).unwrap();
                if body == other_body {
                    return Err("???".to_string());
                }
            }
        }
        self.body.insert(0, new_head);
        return Ok(());
    }
    fn eat_fruit(&mut self) {
        let a = self.body.get(self.lengh - 1).unwrap(); // x: 64 y: 24
        let b = self.body.get(self.lengh - 2).unwrap(); // x: 64 y: 25
        let mut x = a.x;
        let mut y = a.y;
        if a.x == b.x {
            y = a.y * 2 - b.y;
        } else if a.y == b.y {
            x = a.x * 2 - b.x;
        }
        let new_hip = SnakeBody { x, y };
        self.body.push(new_hip);
        self.lengh += 1;
    }
}

impl Fruit {
    fn new(field_width: usize, field_height: usize, skin: char) -> Self {
        let mut rnd = rand::thread_rng();
        let x = rnd.gen_range(1..field_width - 1);
        let y = rnd.gen_range(1..field_height - 1);
        return Fruit {
            x,
            y,
            field_width,
            field_height,
            skin,
        };
    }
    fn move_random(&mut self) {
        let mut rnd = rand::thread_rng();
        self.x = rnd.gen_range(1..self.field_width - 1);
        self.y = rnd.gen_range(1..self.field_height - 1)
    }
}

fn main() {
    let width = 32;
    let height = 32;

    let mut game_field = GameField::new(width, height, '#', ' ');
    game_field.init();
    let mut snake = Snake::new('O', 3, width, height);
    snake.init(12, 12);
    let read_direction_thread = spawn_read_direction_thread();
    let mut fruit = Fruit::new(width, height, '@');

    loop {
        game_field.tile_reset();

        let snake_lengh = snake.get_lengh();
        for i in 0..snake_lengh {
            let body = snake.get_body(i).unwrap();
            game_field.tile_update(body.x, body.y, snake.skin);
        }

        let snake_head = snake.get_body(0).unwrap();
        if is_touch_fruit(snake_head, &fruit) {
            snake.eat_fruit();
            fruit.move_random();
        }
        game_field.tile_update(fruit.x, fruit.y, fruit.skin);

        game_field.render();

        if let Ok(Some(direction)) = read_direction_thread.try_recv() {
            snake.set_direction(direction);
        }

        let result = snake.move_body();
        if let Err(_) = result {
            break;
        }

        thread::sleep(time::Duration::from_millis(100));
    }
}

fn spawn_read_direction_thread() -> mpsc::Receiver<Option<Direction>> {
    let (tx, rx) = mpsc::channel::<Option<Direction>>();
    thread::spawn(move || {
        let options = ReaderOptions::default();
        let reader = TermReader::open_stdin(&options).unwrap();
        loop {
            let direction = match reader.read_bytes(3).unwrap()[2] {
                65 => Some(Direction::Up),
                66 => Some(Direction::Down),
                68 => Some(Direction::Left),
                67 => Some(Direction::Right),
                _ => None,
            };
            tx.send(direction).unwrap();
        }
    });
    return rx;
}

fn is_touch_fruit(snake_head: &SnakeBody, fruit: &Fruit) -> bool {
    if snake_head.x == fruit.x && snake_head.y == fruit.y {
        return true;
    } else {
        return false;
    }
}
