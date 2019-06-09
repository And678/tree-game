use rand::Rng;
use termion::raw::IntoRawMode;
use termion::async_stdin;
use std::io::{Read, Write, Cursor, stdout};
use std::thread;
use std::time::Duration;
use termion::{color, cursor, clear};
use rodio::source::Source;
#[derive(PartialEq, Copy, Clone)]
enum Actions {
    Left,
    Right,
    Nothing,
    Restart,
    Quit,
}

const TOTAL_TIME: u64 = 10_000;
const TIME_REDUCE_AMOUNT: u64 = 1000;
const TREE_LENGTH: u32 = 20;
const TREE_BONUS: i32 = 25;
const FRAME_TIME: u64 = 50;

fn main() {
    let game_over_sound: &'static [u8] = include_bytes!("./game_over.wav");
    let tree_sound: &'static [u8] = include_bytes!("./tree.wav");
    let wood_sound: &'static [u8] = include_bytes!("./wood.wav");
    let sound_device = rodio::default_output_device().unwrap();

    let mut generator = rand::thread_rng();
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let mut stdin = async_stdin();
    let mut level = gen_tree(&mut generator, TREE_LENGTH);
    let mut score = 0;
    let mut trees = 0;
    let mut total_time = TOTAL_TIME;
    let mut current_time: u64 = 0;
    let mut last_action = Actions::Nothing;
    let mut game_in_progress = true;

    render(&mut stdout, &level, score, trees, last_action);
    loop {
        let action = read_key(&mut stdin);
        if action == Actions::Quit {
            clear(&mut stdout);
            break;
        } 
        if action == Actions::Restart {
            level = gen_tree(&mut generator, TREE_LENGTH);
            score = 0;
            trees = 0;
            total_time = TOTAL_TIME;
            current_time = 0;
            game_in_progress = true;
            render(&mut stdout, &level, score, trees, last_action);
            continue;
        }
        if game_in_progress {
            if current_time >= total_time {
                render_gameover(&mut stdout);
                play_sound(&sound_device, game_over_sound);
                game_in_progress = false;
                continue;    
            }
            if action != Actions::Nothing {
                last_action = action;     
                let result = handle_action(action, level[0]);
                if result {
                    level.remove(0);
                    if level.len() == 0 {
                        play_sound(&sound_device, tree_sound);
                        level = gen_tree(&mut generator, TREE_LENGTH);
                        score += TREE_BONUS;
                        trees += 1;
                        total_time -= TIME_REDUCE_AMOUNT;
                        current_time = 0;
                    } else {
                        play_sound(&sound_device, wood_sound);
                        score += 1;
                    }
                    render(&mut stdout, &level, score, trees, last_action);
                } else {
                    render_gameover(&mut stdout);
                    play_sound(&sound_device, game_over_sound);
                    game_in_progress = false;
                    continue;
                }
            }
            render_timer(&mut stdout, total_time, current_time);
            current_time += FRAME_TIME;
        }
        thread::sleep(Duration::from_millis(FRAME_TIME));
        stdout.flush().unwrap();
    }
}
fn play_sound(device: &rodio::Device, sound: &'static [u8]) {
    let source = rodio::Decoder::new(Cursor::new(sound)).unwrap();
    rodio::play_raw(device, source.convert_samples());
}

fn clear(stdout: &mut Write) {
    let (_, y) = termion::terminal_size().unwrap();
    writeln!(stdout, "{}{}{}",
                color::Bg(color::Reset),
                color::Fg(color::Reset),
                cursor::Goto(0, y)
            ).unwrap();
}

fn render_timer(stdout: &mut Write, total: u64, current: u64) {
    let (x, _) = termion::terminal_size().unwrap();
    if x > 8 && total > current {
        let total_length = x - 8;
        let percent: f64 = current as f64 / total as f64;
        let elapsed: u16 = (total_length as f64 * percent) as u16;
        let rest = total_length - elapsed;
        write!(stdout, "{}{}",
                cursor::Goto(4, 4),
                color::Bg(color::Green)
            ).unwrap();
        for _ in 0..elapsed {
            write!(stdout, " ").unwrap();
        } 
        write!(stdout, "{}",
                color::Bg(color::White)
            ).unwrap();
        for _ in 0..rest {
            write!(stdout, " ").unwrap();
        }
    }
}

fn render(stdout: &mut Write, level: &Vec<i32>, score: i32, trees: i32, last_action: Actions) {
    let (x, y) = termion::terminal_size().unwrap();
    let middle_x = x / 2;

    write!(stdout, "{}", color::Bg(color::Blue)).unwrap();
    write!(stdout, "{}", clear::All).unwrap();
    write!(stdout, "{}{}  Score: {};  Trees: {}", 
        cursor::Goto(2, 2),
        color::Fg(color::White),
        score, trees)
    .unwrap();
    write!(stdout, "{}", color::Bg(color::Rgb(128, 0, 0))).unwrap();
    write!(stdout, "{}    ", cursor::Goto(middle_x - 2, y  - 1)).unwrap();
    write!(stdout, "{}    ", cursor::Goto(middle_x - 2, y)).unwrap();
    for (i, &num) in level.iter().enumerate() {
        write!(stdout, "{}", color::Bg(color::Rgb(128, 0, 0))).unwrap();
        let height: u16 = i as u16 * 2 + 2;
        if height >= y {
            break;
        }
        write!(stdout, "{}    ", cursor::Goto(middle_x - 2, y - height - 1)).unwrap();
        write!(stdout, "{}    ", cursor::Goto(middle_x - 2, y - height)).unwrap();
        write!(stdout, "{}", color::Bg(color::Green)).unwrap();
        if num == 2 {
            write!(stdout, "{}    ", cursor::Goto(middle_x - 6, y - height - 1)).unwrap();
            write!(stdout, "{}    ", cursor::Goto(middle_x - 6, y - height)).unwrap();
        } else if num == 1 {
            write!(stdout, "{}    ", cursor::Goto(middle_x + 2, y - height - 1)).unwrap();
            write!(stdout, "{}    ", cursor::Goto(middle_x + 2, y - height)).unwrap();
        }
    }
    if last_action == Actions::Left {
        write!(stdout, "{}{} ", 
            cursor::Goto(middle_x - 4, y),
            color::Bg(color::White))
        .unwrap();
    }
    else if last_action == Actions::Right {
        write!(stdout, "{}{} ", 
            cursor::Goto(middle_x + 3, y),
            color::Bg(color::White))
        .unwrap();
    }
}

fn render_gameover(stdout: &mut Write) {
    write!(stdout, "{}{}", color::Bg(color::Red), color::Fg(color::White)).unwrap();
    let (x, y) = termion::terminal_size().unwrap();
    let middle_x = x / 2;
    let middle_y = y / 2;
    write!(stdout, "{}                  ", cursor::Goto(middle_x - 9,    middle_y - 1)).unwrap();
    write!(stdout, "{}    GAME OVER!    ", cursor::Goto(middle_x - 9,        middle_y)).unwrap();
    write!(stdout, "{}                  ", cursor::Goto(middle_x - 9,    middle_y + 1)).unwrap();
    write!(stdout, "{} r - repeat       ", cursor::Goto(middle_x - 9,    middle_y + 2)).unwrap();
    write!(stdout, "{} q - quit         ", cursor::Goto(middle_x - 9,    middle_y + 3)).unwrap();
}

fn read_key(reader: &mut Read) -> Actions {
    let mut result = Vec::new();
    reader.read_to_end(&mut result).unwrap();
    match result.as_slice() {
        [27, 91, 68] => Actions::Left,
        [27, 91, 67] => Actions::Right,
        [114] => Actions::Restart,
        [113] => Actions::Quit,
        [3] => Actions::Quit,
        _ => Actions::Nothing,
    }
}

fn handle_action(action: Actions, place: i32) -> bool{
    (action == Actions::Left && place == 1) || 
    (action == Actions::Right && place == 2) ||
    place == 0
}


fn gen_tree(generator: &mut rand::rngs::ThreadRng, length: u32) -> Vec<i32> {
    let mut level = Vec::new();
    for _ in 0..length {
        level.push(generator.gen_range(0, 3));
    }
    level
}