use std::io;
use rand::seq::SliceRandom;

const TERMINAL_COLOR_GREEN:   &str = "\x1B[32m";
const TERMINAL_COLOR_RED:     &str = "\x1B[31m";
const TERMINAL_COLOR_BLUE:    &str = "\x1B[34m";
const TERMINAL_COLOR_DEFAULT: &str = "\x1B[0m";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Color { // TODO make this have Red and Blue more closely tied than Green somehow
    Green,
    Red,
    Blue,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    B, // Bigs are stored by their corner with the smallest coordinates (closest to a0)
    Goal,
    Number,
}
#[derive(Clone, Copy, Debug)]
struct Piece {
    color: Color,
    kind: Kind,
    health: u8,
    pos: Square,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct Square {
    rank: usize,
    file: usize,
}
#[derive(Clone, Copy, Debug)]
struct Move {
    start: Square,
    end: Square,
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
                rendered_board[rendered_rank][rendered_file].appearance = (b'0' + piece.health) as char;
            },
        }
    }
    for rendered_rank in &rendered_board {
        for rendered_piece in rendered_rank {
            match rendered_piece.color {
                Color::Green => print!("{}", TERMINAL_COLOR_GREEN),
                Color::Red   => print!("{}", TERMINAL_COLOR_RED),
                Color::Blue  => print!("{}", TERMINAL_COLOR_BLUE), 
            }
            print!("{}{}", rendered_piece.appearance, TERMINAL_COLOR_DEFAULT);
        }
        println!();
    }
}

fn is_piece_blocking_square(potential_blocker: &Piece, from: &Piece, square: Square) -> bool {
    if potential_blocker.kind == Kind::B {
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
    } else {
        potential_blocker.pos == square
    }
}

fn is_square_inhabitable_for(pieces: &[&mut Piece], piece: &Piece, square: Square) -> bool {
    if square.rank >= BOARD_SIZE || square.file >= BOARD_SIZE {
        return false;
    } else {
        for potential_blocker in pieces {
            if is_piece_blocking_square(potential_blocker, piece, square) {
                return false;
            }
        }
    }
    true
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

fn list_possible_moves(pieces: &[&mut Piece], piece: &Piece, moves: Vec<[i8; 2]>) -> Vec<Move> {
    moves.iter().map(|x| Move {
        start: piece.pos,
        end: Square {
            rank: add_usize_int(piece.pos.rank, x[0]),
            file: add_usize_int(piece.pos.file, x[1]),
        }
    }).filter(|mv| is_square_inhabitable_for(&pieces, piece, mv.end)).collect()
}

fn find_possible_moves(pieces: &[&mut Piece], turn: Color) -> Vec<Vec<Move>> {
    pieces.iter().filter(|piece| piece.color == turn).map(|piece| 
        list_possible_moves(pieces, piece, get_move_directions(piece))
    ).collect()
}

fn ai_player_get_move(pieces: &[&mut Piece], ai_player: Color) -> Option<Move> {
    find_possible_moves(pieces, ai_player).concat().choose(&mut rand::thread_rng()).copied()
}

const HUMAN_PLAYER: Color = Color::Red;
fn main() {
    let mut curr_game_turn = Color::Red;
    //let curr_tic_tac_toe_turn = Color::Blue;
    let mut all_pieces: Vec<Piece> = vec![
        Piece {color: Color::Red,   kind: Kind::Number, health: 1, pos: Square {rank: 0, file: 0}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 0, file: 1}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 3, pos: Square {rank: 0, file: 2}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 5, pos: Square {rank: 0, file: 3}},
        Piece {color: Color::Red,   kind: Kind::Goal,   health: 1, pos: Square {rank: 0, file: 4}},
        Piece {color: Color::Red,   kind: Kind::Goal,   health: 1, pos: Square {rank: 0, file: 5}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 5, pos: Square {rank: 0, file: 6}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 3, pos: Square {rank: 0, file: 7}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 0, file: 8}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 1, pos: Square {rank: 0, file: 9}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 3, pos: Square {rank: 1, file: 0}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 1, file: 1}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 1, file: 2}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 1, file: 7}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 2, pos: Square {rank: 1, file: 8}},
        Piece {color: Color::Red,   kind: Kind::Number, health: 3, pos: Square {rank: 1, file: 9}},
        Piece {color: Color::Red,   kind: Kind::B,      health: 1, pos: Square {rank: 2, file: 1}},
        Piece {color: Color::Red,   kind: Kind::B,      health: 1, pos: Square {rank: 2, file: 7}},
    
        Piece {color: Color::Green, kind: Kind::Number, health: 0, pos: Square {rank: 4, file: 4}},
        Piece {color: Color::Green, kind: Kind::Number, health: 0, pos: Square {rank: 4, file: 5}},
        Piece {color: Color::Green, kind: Kind::Number, health: 0, pos: Square {rank: 5, file: 4}},
        Piece {color: Color::Green, kind: Kind::Number, health: 0, pos: Square {rank: 5, file: 5}},
    
        Piece {color: Color::Blue,  kind: Kind::Number, health: 1, pos: Square {rank: 9, file: 0}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 9, file: 1}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 3, pos: Square {rank: 9, file: 2}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 5, pos: Square {rank: 9, file: 3}},
        Piece {color: Color::Blue,  kind: Kind::Goal,   health: 1, pos: Square {rank: 9, file: 4}},
        Piece {color: Color::Blue,  kind: Kind::Goal,   health: 1, pos: Square {rank: 9, file: 5}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 5, pos: Square {rank: 9, file: 6}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 3, pos: Square {rank: 9, file: 7}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 9, file: 8}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 1, pos: Square {rank: 9, file: 9}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 3, pos: Square {rank: 8, file: 0}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 8, file: 1}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 8, file: 2}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 8, file: 7}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 2, pos: Square {rank: 8, file: 8}},
        Piece {color: Color::Blue,  kind: Kind::Number, health: 3, pos: Square {rank: 8, file: 9}},
        Piece {color: Color::Blue,  kind: Kind::B,      health: 1, pos: Square {rank: 6, file: 1}},
        Piece {color: Color::Blue,  kind: Kind::B,      health: 1, pos: Square {rank: 6, file: 7}},
    ];
    let mut pieces = all_pieces.iter_mut().collect::<Vec<&mut Piece>>();
    println!("Welcome to this game. You, the human, are playing as {:?}.", HUMAN_PLAYER);
    loop { // main game loop
        render_pieces(&pieces);

        //println!("{:?}", find_possible_moves(&pieces, curr_game_turn));
        if curr_game_turn == HUMAN_PLAYER {
            println!("It's your turn ({:?})!", curr_game_turn);
            'get_player_input: loop {
                println!("Move piece at which rank? ");
                let mut rank_moved = String::new();
                io::stdin().read_line(&mut rank_moved).expect("Failed to read line");
                let rank_moved: usize = rank_moved.trim().parse().expect("Not a number");
                println!("At which file? ");
                let mut file_moved = String::new();
                io::stdin().read_line(&mut file_moved).expect("Failed to read line");
                let file_moved: usize = file_moved.trim().parse().expect("Not a number");
                let mut move_even_made = false;
                for index in 0..pieces.len() {
                    let piece = &pieces[index];
                    if piece.pos == (Square{rank: rank_moved, file: file_moved}) {
                        println!("Choosing to move {:?}", piece);
                        let possible_moves = list_possible_moves(&pieces, piece, get_move_directions(piece));
                        let move_made: &Move;
                        match possible_moves.len() {
                            0 => {
                                println!("This piece has no legal moves!");
                                continue 'get_player_input;
                            }
                            1 => {
                                move_made = possible_moves.get(0).unwrap();
                                println!("Piece has one single legal move: {:?}. Undergoing that move", move_made);
                                move_even_made = true;
                            },
                            _ => {
                                loop {
                                    println!("Which move of {:?}?", possible_moves);
                                    let mut index_chosen = String::new();
                                    io::stdin().read_line(&mut index_chosen).expect("Failed to read line");
                                    let index_chosen: usize = index_chosen.trim().parse().expect("Not a real index");
                                    match possible_moves.get(index_chosen) {
                                        Some(i) => {
                                            move_made = i;
                                            move_even_made = true;
                                            break;
                                        },
                                        None => {
                                            continue;
                                        },
                                    }
                                }
                            }
                        }
                        // Actually make the move
                        let piece = &mut pieces[index];
                        piece.pos = move_made.end;
                        break;
                    }
                }
                // TODO if you input an index that isn't a piece, it will just loop again, which is weird
            }
        } else { // AI player's turn
            println!("It's the AI player's turn now ({:?}).", curr_game_turn);
            // Actually make the move
            // TODO this code is duplicated
            let move_made = ai_player_get_move(&pieces, curr_game_turn);
            match move_made {
                Some(m) => {
                    for piece in &mut pieces {
                        if piece.pos == m.start {
                            piece.pos = m.end;
                            break;
                        }
                    }
                },
                None => panic!("The enemy player is stalemated! Something happens! This shouldn't be a panic!"),
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
