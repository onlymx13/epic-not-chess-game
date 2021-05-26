use std::io;

const COLOR_GREEN:   &str = "\x1B[32m";
const COLOR_RED:     &str = "\x1B[31m";
const COLOR_BLUE:    &str = "\x1B[34m";
const COLOR_DEFAULT: &str = "\x1B[0m";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
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
#[derive(Debug)]
struct Move<'a> {
    piece: &'a Piece,
    square: Square,
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
        rendered_board[piece.pos.rank][piece.pos.file].color = piece.color;
        match piece.kind {
            Kind::B => {
                rendered_board[piece.pos.rank][piece.pos.file].appearance = 'B';
                rendered_board[piece.pos.rank + 1][piece.pos.file].appearance = 'B';
                rendered_board[piece.pos.rank + 1][piece.pos.file].color = piece.color;
                rendered_board[piece.pos.rank][piece.pos.file + 1].appearance = 'B';
                rendered_board[piece.pos.rank][piece.pos.file + 1].color = piece.color;
                rendered_board[piece.pos.rank + 1][piece.pos.file + 1].appearance = 'B';
                rendered_board[piece.pos.rank + 1][piece.pos.file + 1].color = piece.color;
            },
            Kind::Goal => {
                rendered_board[piece.pos.rank][piece.pos.file].appearance = 'X';
            },
            Kind::Number => {
                rendered_board[piece.pos.rank][piece.pos.file].appearance = (b'0' + piece.health) as char;
            },
        }
    }
    for rendered_rank in &rendered_board {
        for rendered_piece in rendered_rank {
            match rendered_piece.color {
                Color::Green => print!("{}", COLOR_GREEN),
                Color::Red   => print!("{}", COLOR_RED),
                Color::Blue  => print!("{}", COLOR_BLUE), 
            }
            print!("{}{}", rendered_piece.appearance, COLOR_DEFAULT);
        }
        println!();
    }
}

fn is_piece_on_square(piece: &Piece, square: Square) -> bool {
    if piece.kind == Kind::B {
        (piece.pos == square) ||
        (Square {
            rank: piece.pos.rank + 1,
            file: piece.pos.file,
        } == square) ||
        (Square {
            rank: piece.pos.rank,
            file: piece.pos.file + 1,
        } == square) ||
        (Square {
            rank: piece.pos.rank + 1,
            file: piece.pos.file + 1,
        } == square)
    } else {
        piece.pos == square
    }
}

fn is_square_inhabitable(pieces: &[&mut Piece], square: Square) -> bool {
    if square.rank >= BOARD_SIZE || square.file >= BOARD_SIZE {
        return false;
    } else {
        for piece in pieces {
            // A big can't move, because it inhabits its own future position...
            if is_piece_on_square(piece, square) {
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
        BOARD_SIZE
    }
}

fn list_possible_moves<'a>(pieces: &[&mut Piece], piece: &'a Piece, moves: Vec<[i8; 2]>) -> Vec<Move<'a>> {
    moves.iter().map(|x| Move {
        piece,
        square: Square {
            rank: add_usize_int(piece.pos.rank, x[0]),
            file: add_usize_int(piece.pos.file, x[1]),
        }
    }).filter(|mv| is_square_inhabitable(&pieces, mv.square)).collect()
}

fn find_possible_moves<'a>(pieces: &'a [&mut Piece], turn: Color) -> Vec<Vec<Move<'a>>> {
    pieces.iter().filter(|piece| piece.color == turn).map(|piece| 
    match piece.kind {
        Kind::B => list_possible_moves(pieces, &piece, vec![[1, 0], [-1, 0], [0, 1], [0, -1]]),
        Kind::Goal => Vec::new(),
        Kind::Number => {
            match piece.health {
                5 => {
                    list_possible_moves(pieces, &piece, vec![
                        [-2, -2], [-2, -1], [-2, 0], [-2, 1], [-2, 2],
                        [-1, -2], [-1, -1], [-1, 0], [-1, 1], [-1, 2],
                        [ 0, -2], [ 0, -1],          [ 0, 1], [ 0, 2],
                        [ 1, -2], [ 1, -1], [ 1, 0], [ 1, 1], [ 1, 2],
                        [ 2, -2], [ 2, -1], [ 2, 0], [ 2, 1], [ 2, 2],
                    ])
                },
                4 => list_possible_moves(pieces, &piece, vec![[-2, -2], [-2, 2], [2, -2], [2, 2], [0, 2], [-2, 0], [0, -2], [2, 0], [-1, -1], [-1, 1], [1, 1], [1, -1]]),
                3 => list_possible_moves(pieces, &piece, vec![[-1, -1], [-1, 0], [-1, 1], [0, 1], [1, 1], [1, 0], [1, -1], [0, -1]]),
                2 => list_possible_moves(pieces, &piece, vec![[0, 1], [-1, 0], [0, -1], [1, 0]]),
                1 => {
                    if piece.color == Color::Red {
                        list_possible_moves(pieces, &piece, vec![[0, 1], [0, -1], [1, 0]])
                    } else {
                        list_possible_moves(pieces, &piece, vec![[0, 1], [-1, 0], [0, -1]])
                    }
                },
                _ => Vec::new(),
            }
        }
    }).collect()
}

fn main() {
    let curr_game_turn = Color::Red;
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
    let pieces = all_pieces.iter_mut().collect::<Vec<&mut Piece>>();

    render_pieces(&pieces);
    println!("{:?}", find_possible_moves(&pieces, curr_game_turn));
    
}