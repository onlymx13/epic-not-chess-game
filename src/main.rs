use std::cmp;
use std::fmt;
use std::io;
use rand::seq::SliceRandom;

const TERMINAL_COLOR_GREEN:   &str = "\x1B[32m";
const TERMINAL_COLOR_RED:     &str = "\x1B[31m";
const TERMINAL_COLOR_BLUE:    &str = "\x1B[34m";
const TERMINAL_COLOR_DEFAULT: &str = "\x1B[0m";

#[derive(Clone, Copy, PartialEq)]
enum Color { // TODO make this have Red and Blue more closely tied than Green somehow
    Green,
    Red,
    Blue,
}
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Color::Green => "Green",
            Color::Red => "Red",
            Color::Blue => "Blue",
        })
    }
}
#[derive(Clone, Copy)]
enum Kind {
    B, // Bigs are stored by their corner with the smallest coordinates (closest to a0)
    Goal,
    Number,
}
impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Kind::B => "B",
            Kind::Goal => "goal",
            Kind::Number => "normal",
        })
    }
}
#[derive(Clone, Copy)]
// It'd probably be easier to just keep track of the board state and not look at pieces' positions.
struct Piece {
    color: Color,
    kind: Kind,
    health: i8,
    pos: Square,
    delete: bool,
}
impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            Kind::Goal => write!(f, "Goal"),
            _ => write!(f, "{} {} piece ({} health) on {}", self.color, self.kind, self.health, self.pos),
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
struct Square {
    rank: usize,
    file: usize,
}
impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.rank, self.file)
    }
}
#[derive(Clone, Copy)]
enum Action {
    Move(Move),
    Explosion(Square),
}
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Action::Move(m) => format!("{}", m),
            Action::Explosion(s) => format!("Explosion on {}", s),
        })
    }
}
#[derive(Clone, Copy)]
struct Move {
    start: Square,
    end: Square,
}
impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Move {} to {}", self.start, self.end)
    }
}
#[derive(Clone, Copy)]
struct RenderedPiece {
    color: Color,
    appearance: char,
}
const BOARD_SIZE: usize = 10;

const NOT_A_RENDERED_PIECE: RenderedPiece = RenderedPiece {
    color: Color::Green,
    appearance: ' ',
};
fn render_pieces(pieces: &[&mut Piece]) {
    let mut rendered_board = [[NOT_A_RENDERED_PIECE; BOARD_SIZE]; BOARD_SIZE];
    for piece in pieces {
        let rendered_rank = match HUMAN_PLAYER { // Render the board upside-down as Red
            Color::Red => BOARD_SIZE - piece.pos.rank - 1,
            Color::Blue => piece.pos.rank,
            Color::Green => 0,
        };
        let rendered_file = match HUMAN_PLAYER { // And backward as Blue
            Color::Red => piece.pos.file,
            Color::Blue => BOARD_SIZE - piece.pos.file - 1,
            Color::Green => 0,
        };
        rendered_board[rendered_rank][rendered_file].color = piece.color;
        match piece.kind {
            Kind::B => {
                match HUMAN_PLAYER {
                    Color::Red => { // Currently working on getting this to work. Make sure it's not L/R mirrored and then fix when human is Blue.
                        rendered_board[rendered_rank - 1][rendered_file].appearance = '╔';
                        rendered_board[rendered_rank - 1][rendered_file].color = piece.color;

                        rendered_board[rendered_rank][rendered_file].appearance = '╚';

                        rendered_board[rendered_rank - 1][rendered_file + 1].appearance = '╗';
                        rendered_board[rendered_rank - 1][rendered_file + 1].color = piece.color;

                        rendered_board[rendered_rank][rendered_file + 1].appearance = '╝';
                        rendered_board[rendered_rank][rendered_file + 1].color = piece.color;
                    },
                    Color::Blue => {
                        //TODO this is wrong
                        rendered_board[rendered_rank][rendered_file].appearance = '╔';
                        rendered_board[rendered_rank + 1][rendered_file].appearance = '╚';
                        rendered_board[rendered_rank + 1][rendered_file].color = piece.color;
                        rendered_board[rendered_rank][rendered_file - 1].appearance = '╗';
                        rendered_board[rendered_rank][rendered_file - 1].color = piece.color;
                        rendered_board[rendered_rank][rendered_file + 1].appearance = '╝';
                        rendered_board[rendered_rank][rendered_file + 1].color = piece.color;
                    },
                    Color::Green => (),
                }
            },
            Kind::Goal => {
                rendered_board[rendered_rank][rendered_file].appearance = 'X';
            },
            Kind::Number => {
                rendered_board[rendered_rank][rendered_file].appearance = (b'0' + (piece.health as u8)) as char;
            },
        }
    }
    let mut counter = match HUMAN_PLAYER {
        Color::Red => 9,
        Color::Blue => 0,
        Color::Green => 0,
    };
    for rendered_rank in &rendered_board {
        print!("{} ", counter);
        match HUMAN_PLAYER {
            Color::Red   => counter -= 1,
            Color::Blue  => counter += 1,
            Color::Green => (),
        }
        for rendered_piece in rendered_rank {
            print!("{}", match rendered_piece.color {
                Color::Green => TERMINAL_COLOR_GREEN,
                Color::Red   => TERMINAL_COLOR_RED,
                Color::Blue  => TERMINAL_COLOR_BLUE,
            });
            print!("{}{}", rendered_piece.appearance, TERMINAL_COLOR_DEFAULT);
        }
        println!();
    }
    println!("rf{}", match HUMAN_PLAYER {
        Color::Red  => "0123456789",
        Color::Blue => "9876543210",
        Color::Green => "",
    });
}

fn does_piece_block_square(potential_blocker: &Piece, square: Square) -> bool {
    match potential_blocker.kind {
        Kind::B => {
            (potential_blocker.pos == square) ||
            (Square {
                rank: potential_blocker.pos.rank + 1,
                file: potential_blocker.pos.file,
            } == square) ||
            (Square {
                rank: potential_blocker.pos.rank,
                file: potential_blocker.pos.file + 1,
            } == square) ||
            (Square {
                rank: potential_blocker.pos.rank + 1,
                file: potential_blocker.pos.file + 1,
            } == square)
        },
    Kind::Number => potential_blocker.pos == square,
    Kind::Goal => false,
    }
}

fn is_piece_blocking_square_from(potential_blocker: &Piece, from: &Piece, square: Square) -> bool {
    if potential_blocker.color != from.color {
        // Only pieces of your own color can block you.
        return false;
    }
    match potential_blocker.kind {
        Kind::B => {
            if potential_blocker.pos == from.pos {
                // B's shouldn't block themselves
                false
            } else {
                (potential_blocker.pos == square) ||
                (Square {
                    rank: potential_blocker.pos.rank + 1,
                    file: potential_blocker.pos.file,
                } == square) ||
                (Square {
                    rank: potential_blocker.pos.rank,
                    file: potential_blocker.pos.file + 1,
                } == square) ||
                (Square {
                    rank: potential_blocker.pos.rank + 1,
                    file: potential_blocker.pos.file + 1,
                } == square)
            }
        },
        Kind::Number => potential_blocker.pos == square,
        Kind::Goal => false,
    }
}

fn is_square_inhabitable_for(pieces: &[&mut Piece], piece: &Piece, square: Square) -> bool {
    if square.rank >= BOARD_SIZE || square.file >= BOARD_SIZE {
        return false;
    } else {
        for potential_blocker in pieces {
            if is_piece_blocking_square_from(potential_blocker, piece, square) {
                return false;
            }
        }
    }
    true
}

fn can_piece_move_to(pieces: &[&mut Piece], piece: &Piece, square: Square) -> bool {
    match piece.kind {
        Kind::B => {
            is_square_inhabitable_for(pieces, piece, square) &&
            is_square_inhabitable_for(pieces, piece, Square{rank: square.rank + 1, file: square.file}) &&
            is_square_inhabitable_for(pieces, piece, Square{rank: square.rank, file: square.file + 1}) &&
            is_square_inhabitable_for(pieces, piece, Square{rank: square.rank + 1, file: square.file + 1})
        },
        Kind::Number => is_square_inhabitable_for(pieces, piece, square),
        Kind::Goal => false,
    }
}

fn add_usize_int(a: usize, b: i8) -> usize {
    if a as i8 + b >= 0 {
        (a as i8 + b) as usize
    } else {
        BOARD_SIZE // represents uninhabitable square
    }
}

fn get_move_directions(piece: &Piece) -> Vec<[i8; 2]> {
    match piece.kind {
        Kind::B => vec![[1, 0], [-1, 0], [0, 1], [0, -1]],
        Kind::Goal => Vec::new(),
        Kind::Number => {
            match piece.health {
                5 => vec![
                    [-2, -2], [-2, -1], [-2, 0], [-2, 1], [-2, 2],
                    [-1, -2], [-1, -1], [-1, 0], [-1, 1], [-1, 2],
                    [ 0, -2], [ 0, -1],          [ 0, 1], [ 0, 2],
                    [ 1, -2], [ 1, -1], [ 1, 0], [ 1, 1], [ 1, 2],
                    [ 2, -2], [ 2, -1], [ 2, 0], [ 2, 1], [ 2, 2],
                ],
                4 => vec![[-2, -2], [-2, 2], [2, -2], [2, 2], [0, 2], [-2, 0], [0, -2], [2, 0], [-1, -1], [-1, 1], [1, 1], [1, -1]],
                3 => vec![[-1, -1], [-1, 0], [-1, 1], [0, 1], [1, 1], [1, 0], [1, -1], [0, -1]],
                2 => vec![[0, 1], [-1, 0], [0, -1], [1, 0]],
                1 => match piece.color {
                    Color::Red => vec![[0, 1], [0, -1], [1, 0]],
                    Color::Blue => vec![[0, 1], [-1, 0], [0, -1]],
                    Color::Green => Vec::new(),
                },
                _ => Vec::new(),
            }
        }
    }
}

fn list_possible_moves(pieces: &[&mut Piece], piece: &Piece, moves: Vec<[i8; 2]>) -> Vec<Action> {
    if let Kind::Goal = piece.kind {
        return Vec::new();
    }
    let mut ret: Vec<Action> = moves.iter().map(|x| Move {
        start: piece.pos,
        end: Square {
            rank: add_usize_int(piece.pos.rank, x[0]),
            file: add_usize_int(piece.pos.file, x[1]),
        }
    }).filter(|mv| can_piece_move_to(&pieces, piece, mv.end)).map(Action::Move).collect();
    if let Kind::B = piece.kind {
        ret.push(Action::Explosion(piece.pos));
    }
    ret
}

fn find_possible_moves(pieces: &[&mut Piece], turn: Color) -> Vec<Vec<Action>> {
    pieces.iter().filter(|piece| piece.color == turn).map(|piece| 
        list_possible_moves(pieces, piece, get_move_directions(piece))
    ).collect()
}

fn ai_player_get_move(pieces: &[&mut Piece], ai_player: Color) -> Option<Action> {
    find_possible_moves(pieces, ai_player).concat().choose(&mut rand::thread_rng()).copied()
}

fn did_player_win(pieces: &[&mut Piece], player: Color) -> bool {
    let goal1;
    let goal2;
    match player {
        Color::Blue => {
            // Positions of Red's goals
            goal1 = Square{rank: 0, file: 4};
            goal2 = Square{rank: 0, file: 5};
        },
        _ => {
            // Positions of Blue's goals
            goal1 = Square{rank: 9, file: 4};
            goal2 = Square{rank: 9, file: 5};
        },
    }
    let mut goal1_reached = false;
    let mut goal2_reached = false;
    for piece in pieces {
        if player == piece.color {
            // Only your own pieces count toward your win
            if does_piece_block_square(piece, goal1) {
                goal1_reached = true;
            }
            if does_piece_block_square(piece, goal2) {
                goal2_reached = true;
            }
        }
    }
    goal1_reached && goal2_reached
}

const HUMAN_PLAYER: Color = Color::Red;
fn main() {
    let mut curr_game_turn = Color::Red;
    //let curr_tic_tac_toe_turn = Color::Blue;
    let mut all_pieces: Vec<Piece> = vec![
        Piece {color: Color::Red,   kind: Kind::Number, health: 1, pos: Square {rank: 0, file: 0}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 0, file: 1}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 3, pos: Square {rank: 0, file: 2}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 5, pos: Square {rank: 0, file: 3}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Goal,   health: 1, pos: Square {rank: 0, file: 4}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Goal,   health: 1, pos: Square {rank: 0, file: 5}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 5, pos: Square {rank: 0, file: 6}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 3, pos: Square {rank: 0, file: 7}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 0, file: 8}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 1, pos: Square {rank: 0, file: 9}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 3, pos: Square {rank: 1, file: 0}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 1, file: 1}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 1, file: 2}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 1, file: 7}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 1, file: 8}, delete: false},
        Piece {color: Color::Red,   kind: Kind::Number, health: 3, pos: Square {rank: 1, file: 9}, delete: false},
        Piece {color: Color::Red,   kind: Kind::B,      health: 4, pos: Square {rank: 2, file: 1}, delete: false},
        Piece {color: Color::Red,   kind: Kind::B,      health: 4, pos: Square {rank: 2, file: 7}, delete: false},
    
        Piece {color: Color::Green, kind: Kind::Number, health: 0, pos: Square {rank: 4, file: 4}, delete: false},
        Piece {color: Color::Green, kind: Kind::Number, health: 0, pos: Square {rank: 4, file: 5}, delete: false},
        Piece {color: Color::Green, kind: Kind::Number, health: 0, pos: Square {rank: 5, file: 4}, delete: false},
        Piece {color: Color::Green, kind: Kind::Number, health: 0, pos: Square {rank: 5, file: 5}, delete: false},
    
        Piece {color: Color::Blue,  kind: Kind::Number, health: 1, pos: Square {rank: 9, file: 0}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 9, file: 1}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 3, pos: Square {rank: 9, file: 2}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 5, pos: Square {rank: 9, file: 3}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Goal,   health: 1, pos: Square {rank: 9, file: 4}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Goal,   health: 1, pos: Square {rank: 9, file: 5}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 5, pos: Square {rank: 9, file: 6}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 3, pos: Square {rank: 9, file: 7}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 9, file: 8}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 1, pos: Square {rank: 9, file: 9}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 3, pos: Square {rank: 8, file: 0}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 8, file: 1}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 8, file: 2}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 8, file: 7}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 8, file: 8}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 3, pos: Square {rank: 8, file: 9}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::B,      health: 4, pos: Square {rank: 6, file: 1}, delete: false},
        Piece {color: Color::Blue,  kind: Kind::B,      health: 4, pos: Square {rank: 6, file: 7}, delete: false},
    ];
    let mut pieces = all_pieces.iter_mut().collect::<Vec<&mut Piece>>();
    println!("Welcome to this game. You, the human, are playing as {}.", HUMAN_PLAYER);
    'main_loop: loop {
        // Delete all pieces marked for deletion
        pieces = pieces.into_iter().filter(|piece| !piece.delete).collect();

        //Figure out if anybody won
        let red_won = did_player_win(&pieces, Color::Red);
        let blue_won = did_player_win(&pieces, Color::Blue);
        if red_won {
            println!("Red won!");
            render_pieces(&pieces);
            break;
        }
        if blue_won {
            println!("Blue won!");
            render_pieces(&pieces);
            break;
        }
        render_pieces(&pieces);

        //println!("{:?}", find_possible_moves(&pieces, curr_game_turn));
        if curr_game_turn == HUMAN_PLAYER {
            println!("It's your turn ({})!", curr_game_turn);
            'get_player_input: loop {
                println!("Move piece at which rank? ");
                let mut rank_moved = String::new();
                io::stdin().read_line(&mut rank_moved).expect("Failed to read line");
                let rank_moved: usize = rank_moved.trim().parse().expect("Not a number");
                println!("At which file? ");
                let mut file_moved = String::new();
                io::stdin().read_line(&mut file_moved).expect("Failed to read line");
                let file_moved: usize = file_moved.trim().parse().expect("Not a number");
                for index in 0..pieces.len() {
                    let piece = &pieces[index];
                    if piece.pos == (Square{rank: rank_moved, file: file_moved}) {
                        println!("Choosing to move {}.", piece);
                        if piece.color != curr_game_turn {
                            println!("That piece isn't yours to move!");
                            continue 'get_player_input;
                        }
                        let possible_moves = list_possible_moves(&pieces, piece, get_move_directions(piece));
                        if possible_moves.is_empty() {
                            println!("You have no legal moves! Your opponent wins!");
                            break 'main_loop;
                        }
                        let move_made: &Action;
                        match possible_moves.len() {
                            0 => {
                                println!("This piece has no legal moves!");
                                continue 'get_player_input;
                            }
                            1 => {
                                move_made = possible_moves.get(0).unwrap();
                                println!("Piece has one single legal move: {}. Undergoing that move", move_made);
                            },
                            _ => {
                                loop {
                                    println!("Which move of {}?", possible_moves.iter().fold(String::new(), |a, &m| a + &m.to_string() + ", "));
                                    let mut index_chosen = String::new();
                                    io::stdin().read_line(&mut index_chosen).expect("Failed to read line");
                                    let index_chosen: usize = index_chosen.trim().parse().expect("Not a real index");
                                    match possible_moves.get(index_chosen) {
                                        Some(i) => {
                                            move_made = i;
                                            println!("Making move {}", i);
                                            break;
                                        },
                                        None => {
                                            println!("Index {} isn't a possible move; there were only {}!", index_chosen, possible_moves.len());
                                            continue;
                                        },
                                    }
                                }
                            }
                        }
                        // Actually make the move

                        match move_made {
                            Action::Explosion(sq) => {
                                for offset in &[[-1, -1], [0, -1], [1, -1], [2, -1], [2, 0], [2, 1], [2, 2], [1, 2], [0, 2], [-1, 2], [-1, 1], [-1, 0]] {
                                    for damaged_piece in &mut pieces {
                                        if damaged_piece.pos == (Square {rank: add_usize_int(sq.rank, offset[0]), file: add_usize_int(sq.file ,offset[1])}) {
                                            damaged_piece.health -= 1;
                                            if damaged_piece.health <= 0 {
                                                damaged_piece.delete = true;
                                            }
                                        }
                                    }
                                }
                                // add ones
                                /*for offset in &[[-1, -1], [2, -1], [2, 2], [-1, 2]] {
                                    let spawned_pos = Square {rank: add_usize_int(piece.pos.rank, offset[0]), file: add_usize_int(piece.pos.file, offset[1])};
                                    if is_square_inhabitable_for(&pieces, piece, spawned_pos) {
                                        let mut one = Piece {color: curr_game_turn, kind: Kind::Number, health: 1, pos: spawned_pos, delete: false};
                                    }
                                }*/
                                // Delete the big that exploded
                                let piece = &mut pieces[index];
                                piece.delete = true;
                            },
                            Action::Move(mv) => {
                                let mut damage = pieces[index].health;
                                let mut piece_attacked = false;
                                for attacked_piece in &mut pieces {
                                    if attacked_piece.pos == mv.end {
                                        damage = cmp::min(damage, attacked_piece.health);
                                        piece_attacked = true;
                                        attacked_piece.health -= damage;
                                        if attacked_piece.health <= 0 {
                                            attacked_piece.delete = true;
                                        }
                                    }
                                }
                                let piece = &mut pieces[index];
                                if piece_attacked {
                                    piece.health -= damage;
                                }
                                if piece.health <= 0 {
                                    piece.delete = true;
                                }
                                piece.pos = mv.end;
                                break 'get_player_input;
                            },
                        }

                    }
                }
                // TODO if you input an index that isn't a piece, it will just loop again, which is weird
            }
        } else { // AI player's turn
            println!("It's the AI player's turn now ({}).", curr_game_turn);
            // Actually make the move
            // TODO this code is duplicated
            let move_made = ai_player_get_move(&pieces, curr_game_turn);
            match move_made {
                Some(action) => {
                    println!("The AI player made move {}", action);
                    for index in 0..pieces.len() {
                        if pieces[index].pos == match action {
                            Action::Explosion(sq) => sq,
                            Action::Move(m) => m.start,
                        } {
                            match action {
                                Action::Explosion(sq) => {
                                    let piece = &mut pieces[index];
                                    // Delete the big that exploded
                                    piece.delete = true;
                                    if piece.pos == sq {
                                        for offset in &[[-1, -1], [0, -1], [1, -1], [2, -1], [2, 0], [2, 1], [2, 2], [1, 2], [0, 2], [-1, 2], [-1, 1], [-1, 0]] {
                                            for damaged_piece in &mut pieces {
                                                if damaged_piece.pos == (Square {rank: add_usize_int(sq.rank, offset[0]), file: add_usize_int(sq.file ,offset[1])}) {
                                                    damaged_piece.health -= 1;
                                                    if damaged_piece.health <= 0 {
                                                        damaged_piece.delete = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                Action::Move(mv) => {
                                    if pieces[index].pos == mv.start {
                                        let mut damage = pieces[index].health;
                                        let mut piece_attacked = false;
                                        for attacked_piece in &mut pieces {
                                            if attacked_piece.pos == mv.end {
                                                damage = cmp::min(damage, attacked_piece.health);
                                                piece_attacked = true;
                                                attacked_piece.health -= damage;
                                                if attacked_piece.health <= 0 {
                                                    attacked_piece.delete = true;
                                                }
                                            }
                                        }
                                        let piece = &mut pieces[index];
                                        if piece_attacked {
                                            piece.health -= damage;
                                        }
                                        if piece.health <= 0 {
                                            piece.delete = true;
                                        }
                                        piece.pos = mv.end;
                                        break;
                                    }
                                }
                            }
                            break;
                        }
                    }
                },
                None => {
                    println!("The enemy player has no legal moves, so you win!");
                },
            }
        }
    
        // End of main loop. Switch the current turn.
        curr_game_turn = match curr_game_turn {
            Color::Red => Color::Blue,
            Color::Blue => Color::Red,
            Color::Green => panic!("It's Green's turn somehow"),
        };
    }
}
