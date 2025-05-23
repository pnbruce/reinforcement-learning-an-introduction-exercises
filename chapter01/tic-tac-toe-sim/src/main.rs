use rand;
use std::collections::{hash_map::Entry, HashMap};

use rand::RngCore;

const WINNING_COMBINATIONS: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8], // Rows
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8], // Columns
    [0, 4, 8],
    [2, 4, 6], // Diagonals
];

const DEFAULT_VALUE: f32 = 0.0;

struct Board {
    pub spaces: u32,
}

impl Board {
    fn new() -> Self {
        Board {
            spaces: 0b000000000000000000,
        }
    }

    fn at(&self, index: u32) -> char {
        let x_mask = PlayerMarker::player_mask(&PlayerMarker::X) << (index * 2);
        let o_mask = PlayerMarker::player_mask(&PlayerMarker::O) << (index * 2);
        if self.spaces & x_mask == x_mask {
            'X'
        } else if self.spaces & o_mask == o_mask {
            'O'
        } else {
            ' '
        }
    }

    fn to_string(&self) -> String {
        format!(
            "{}|{}|{}\n-----\n{}|{}|{}\n-----\n{}|{}|{}",
            self.at(0),
            self.at(1),
            self.at(2),
            self.at(3),
            self.at(4),
            self.at(5),
            self.at(6),
            self.at(7),
            self.at(8)
        )
    }

    fn print(&self) {
        println!("{}", self.to_string());
    }

    fn check_winner(&self, player: &PlayerMarker) -> bool {
        let player_char = PlayerMarker::player_char(player);
        WINNING_COMBINATIONS
            .iter()
            .any(|&combo| combo.iter().all(|&i| self.at(i as u32) == player_char))
    }

    fn is_draw(&self) -> bool {
        self.spaces & 0b101010101010101010 == 0b101010101010101010
    }

    fn available(&self, index: usize) -> bool {
        let mask = 0b11 << (index * 2);
        (self.spaces & mask) == 0b0
    }

    fn set(&mut self, index: usize, value: &PlayerMarker) {
        let player_char = PlayerMarker::player_mask(value);
        self.spaces |= player_char << (index * 2);
    }
}

enum PlayerMarker {
    X,
    O,
}

impl PlayerMarker {
    fn player_char(player: &PlayerMarker) -> char {
        match player {
            PlayerMarker::X => 'X',
            PlayerMarker::O => 'O',
        }
    }

    fn player_mask(player: &PlayerMarker) -> u32 {
        match player {
            PlayerMarker::X => 0b11,
            PlayerMarker::O => 0b10,
        }
    }
}

enum Agent {
    Random,
    Human,
    RL(HashMap<u32, f32>, u32),
}

impl Agent {
    fn get_move(&mut self, board: &Board, player: &PlayerMarker) -> usize {
        match self {
            Agent::Random => {
                let available: Vec<usize> = (0..9).filter(|&i| board.available(i)).collect();
                let index = rand::rng().next_u32() as usize % available.len();
                *available.get(index).expect("Board is full")
            }
            Agent::Human => loop {
                board.print();
                println!("{} to move!", PlayerMarker::player_char(player));
                println!("Enter a number between 1 and 9:");
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");
                let move_index: usize = match input.trim().parse::<usize>() {
                    Ok(num) if num >= 1 && num <= 9 => num - 1,
                    _ => {
                        println!("Invalid input. Please enter a number between 1 and 9.");
                        continue;
                    }
                };
                if !board.available(move_index) {
                    println!("That space is taken. Try again.");
                    continue;
                }
                return move_index;
            },
            Agent::RL(q_table, prev_board) => {
                let mut best_move = None;
                let mut best_value = f32::MIN;
                let mut values: Vec<Value> = Vec::new();
                for i in 0..9 {
                    if board.available(i) {
                        let eval_board = board.spaces | (PlayerMarker::player_mask(player) << (i * 2));
                        let value = q_table
                            .get(&eval_board)
                            .unwrap_or(&0.0);
                        values.push(Value::Eval(*value));
                        if *value > best_value {
                            best_value = *value;
                            best_move = Some(i);
                        }
                    } else {
                        values.push(Value::PlayerMarker(board.at(i as u32)));
                    }
                }
                visualize_values(values);
                if rand::rng().next_u32() % 100 == 1 {
                    let available: Vec<usize> = (0..9).filter(|&i| board.available(i)).collect();
                    let index = rand::rng().next_u32() as usize % available.len();
                    return *available.get(index).expect("Board is full");
                }
                update_q(q_table, prev_board, best_value);
                let best_move: usize = best_move.expect("No available moves");
                *prev_board = board.spaces | (PlayerMarker::player_mask(player) << (best_move * 2));
                best_move
            }
        }
    }

    fn report_win(&mut self, player: &PlayerMarker, board: &Board) {
        match self {
            Agent::Random => (),
            Agent::Human => {
                board.print();
                println!("Player {} wins!", PlayerMarker::player_char(&player));
            }
            Agent::RL(q_table, prev_board) => {
                let reward = 1.0;
                update_q(q_table, prev_board, reward);
                *prev_board = 0;
            }
        }
    }

    fn report_draw(&mut self, board: &Board) {
        match self {
            Agent::Random => (),
            Agent::Human => {
                board.print();
                println!("It's a draw!");
            }
            Agent::RL(q_table, prev_board) => {
                let reward = -0.5;
                update_q(q_table, prev_board, reward);
                *prev_board = 0;
                
            }
        }
    }

    fn report_loss(&mut self, player: &PlayerMarker, board: &Board) {
        match self {
            Agent::Random => (),
            Agent::Human => {
                board.print();
                println!("Player {} loses!", PlayerMarker::player_char(&player));
            }
            Agent::RL(q_table, prev_board) => {
                let reward = -1.0;
                update_q(q_table, prev_board, reward);
                *prev_board = 0;
            }
        }
    }
}

fn update_q(q_table: &mut HashMap<u32, f32>, prev_board: &mut u32, reward: f32) {
    let prev_value = q_table.entry(*prev_board);
    match prev_value {
        Entry::Occupied(mut entry) => {
            let prev_reward = *entry.get();
            entry.insert(prev_reward + 0.1 * (reward - prev_reward));
        }
        Entry::Vacant(entry) => {
            entry.insert(DEFAULT_VALUE + 0.1 * (reward - DEFAULT_VALUE));
        }
    }
    // println!("{q_table:?}");
}

enum Value {
    PlayerMarker(char),
    Eval(f32),
}

impl Value {
    fn to_string(&self) -> String {
        match self {
            Value::PlayerMarker(player) => {
                return player.to_string();
            }
            Value::Eval(value) => {
                return value.to_string();
            }
        }
    }
}

enum Result {
    XWin,
    OWin,
    Draw,
}

fn main() {
    let mut random_agent = Agent::Random;
    let mut o_rl = Agent::RL(HashMap::new(), 0);
    let mut x_rl= Agent::RL(HashMap::new(), 0);
    // let mut o_agent = Agent::Random;
    let mut x_wins = 0;
    let mut o_wins = 0;
    let mut draws = 0;
    let games = 100000;

    for _ in 0..games {
        play_game(&mut x_rl, &mut o_rl);
    }

    for _ in 0..games {
        match play_game(&mut random_agent, &mut o_rl) {
            Result::XWin => {
                x_wins += 1;
            }
            Result::OWin => {
                o_wins += 1;
            }
            Result::Draw => {
                draws += 1;
            }
        }
        println!("X wins: {}\t wins: {}\t Draws: {}", x_wins, o_wins, draws);
    }

    for _ in 0..games {
        play_game(&mut Agent::Human, &mut o_rl);
    }
}

fn play_game(x_agent: &mut Agent, o_agent: &mut Agent) -> Result {
    let mut board = Board::new();
    let mut current_player = PlayerMarker::X;
    let mut current_agent = x_agent;
    let mut other_agent = o_agent;

    loop {
        let move_index = current_agent.get_move(&board, &current_player);
        board.set(move_index, &current_player);

        if board.check_winner(&current_player) {
            current_agent.report_win(&current_player, &board);
            other_agent.report_loss(&current_player, &board);
            return match current_player {
                PlayerMarker::X => Result::XWin,
                PlayerMarker::O => Result::OWin,
            };
        }
        if board.is_draw() {
            current_agent.report_draw(&board);
            other_agent.report_draw(&board);
            return Result::Draw;
        }
        current_player = match current_player {
            PlayerMarker::X => PlayerMarker::O,
            PlayerMarker::O => PlayerMarker::X,
        };
        std::mem::swap(&mut current_agent, &mut other_agent);
    }
}

fn visualize_values(values: Vec<Value>) {
    println!(
        "{}|{}|{}\n-----\n{}|{}|{}\n-----\n{}|{}|{}\n",
        values.get(0).expect("msg").to_string(),
        values.get(1).expect("msg").to_string(),
        values.get(2).expect("msg").to_string(),
        values.get(3).expect("msg").to_string(),
        values.get(4).expect("msg").to_string(),
        values.get(5).expect("msg").to_string(),
        values.get(6).expect("msg").to_string(),
        values.get(7).expect("msg").to_string(),
        values.get(8).expect("msg").to_string()
    )
}
