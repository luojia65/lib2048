extern crate lib2048;
extern crate console;

use lib2048::*;
use std::ops::Index;
use std::ops::IndexMut;
use std::io::Write;

static SIZE: (usize, usize) = (4, 4);

struct FrontendBoard(Vec<u8>, TilePos);

impl FrontendBoard {
    fn new(size: impl Into<TilePos>) -> FrontendBoard {
        let size = size.into();
        let mut vec = Vec::new();
        vec.resize(size.size_as_usize(), 0);
        FrontendBoard(vec, size)
    }
}

impl Index<TilePos> for FrontendBoard {
    type Output = u8;

    fn index(&self, pos: TilePos) -> &u8 {
        &self.0[pos.to_usize_index(self.1)]
    }
}

impl IndexMut<TilePos> for FrontendBoard {
    fn index_mut(&mut self, pos: TilePos) -> &mut u8 {
        &mut self.0[pos.to_usize_index(self.1)]
    }
}

fn main() {
    let mut board = Board::new(SIZE);
    let mut frontend_board = FrontendBoard::new(SIZE);
    let mut ds = board.start_game();
    let mut term = console::Term::stdout();
    'main: loop {
        //process last operation
        for d in ds {
            match d {
                Display::Create { pos, value } => {
                    frontend_board[pos] = value;
                },
                Display::CombineInto { a, b, target } => {
                    let value = frontend_board[a];
                    frontend_board[a] = 0;
                    frontend_board[b] = 0;
                    frontend_board[target] = value + 1;
                },
                Display::Move { from, to } => {
                    frontend_board[to] = frontend_board[from];
                    frontend_board[from] = 0;
                },
                Display::GameOver => {
                    writeln!(term, "Game over!");
                    break 'main;
                }
            }
        }
        //print the board
        for i in 0..4 {
            for j in 0..4 {
                write!(term, "{} ", frontend_board[TilePos::from((i, j))]);
            }
            writeln!(term);
        }
        //wait for an operation
        let mut input;
        let ct;
        'input: loop {
            input = String::new();
            std::io::stdin().read_line(&mut input)
                .expect("Failed to read line");
            ct = match input {
                ref s if s == "a\n" => Control::Left,
                ref s if s == "d\n" => Control::Right,
                ref s if s == "s\n" => Control::Down,
                ref s if s == "w\n" => Control::Up,
                _ => {
                    writeln!(term, "invalid input: {}", input);
                    continue 'input
                }
            };
            break
        }
        //store into ds, and in the next loop it will process
        ds = board.control_and_generate(ct);
    }
}
