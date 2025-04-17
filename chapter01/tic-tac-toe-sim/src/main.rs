use rand::RngCore;

const WINNING_COMBINATIONS: [[usize; 3]; 8] = [
    [0, 1, 2], [3, 4, 5], [6, 7, 8], // Rows
    [0, 3, 6], [1, 4, 7], [2, 5, 8], // Columns
    [0, 4, 8], [2, 4, 6],            // Diagonals
];

struct Board {
    pub spaces: u32,
}

impl Board {
    fn new() -> Self {
        Board { spaces: 0b000000000000000000 }
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
            self.at(0), self.at(1), self.at(2),
            self.at(3), self.at(4), self.at(5),
            self.at(6), self.at(7), self.at(8)
        )
    }

    fn print(&self) {
        println!("{}", self.to_string());
    }

    fn check_winner(&self, player: &PlayerMarker) -> bool {
        let player_char = PlayerMarker::player_char(player);
        WINNING_COMBINATIONS.iter().any(|&combo| {
            combo.iter().all(|&i| self.at(i as u32) == player_char)
        })
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
}

impl Agent {
    fn get_move(&self, board: &Board, player: &PlayerMarker) -> usize {
        match self {
            Agent::Random => {
                let available: Vec<usize> = (0..9)
                    .filter(|&i| board.available(i))
                    .collect();
                let index = rand::rng().next_u32() as usize % available.len();
                *available.get(index).expect("Board is full")
            }
            Agent::Human => {
                loop {
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
                }
            }
        }
    }

    fn report_win(&self, player: &PlayerMarker, board: &Board) {
        match self {
            Agent::Random => (),
            Agent::Human => {
                board.print();
                println!("Player {} wins!", PlayerMarker::player_char(&player));
            }
        }
    }

    fn report_draw(&self, board: &Board) {
        match self {
            Agent::Random => (),
            Agent::Human => {
                board.print();
                println!("It's a draw!");
            }
        }
    }

    fn report_loss(&self, player: &PlayerMarker, board: &Board) {
        match self {
            Agent::Random => (),
            Agent::Human => {
                board.print();
                println!("Player {} loses!", PlayerMarker::player_char(&player));
            }
        }
    }
}

enum Result {
    XWin,
    OWin,
    Draw
}

fn main() {
    let x_agent = Agent::Human;
    let o_agent = Agent::Random;
    let mut x_wins = 0;
    let mut o_wins = 0;
    let mut draws = 0;
    let games = 3;
    for _ in 0..games {
        match play_game(&x_agent, &o_agent) {
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
        println!("X wins: {}\t O wins: {}\t Draws: {}", x_wins, o_wins, draws);
    }
}

fn play_game(x_agent: &Agent, o_agent: &Agent) -> Result{
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
