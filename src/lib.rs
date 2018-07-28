extern crate rand;

use std::ops::Index;
use std::ops::IndexMut;

/*
    Example frontend-backend communication

    > go right
    < display move: tile (1, 1) -> (1, 4), ...
      display combine: tile (1, 2) into tile (1, 1)
      display create: tile (3, 3) -> 2, ...
    > go left
    < display ...
    > go up
    < game over

 */

//////////////////////////////////// STRUCTURES ////////////////////////////////////

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct TilePos(usize, usize);

impl From<(usize, usize)> for TilePos {
    fn from(p: (usize, usize)) -> Self {
        TilePos(p.0, p.1)
    }
}

impl TilePos {
    /* When size = (300, 500), maximum pos is (299, 499)
       (0, 0) => 0
       (0, 10) => 10
       (0, 499) => 500
       (1, 0) => 501
       (1, 499) => 1000,
       (299, 499) => 300 * 500 - 1
    */
    pub fn to_usize_index(&self, size: TilePos) -> usize {
        self.0 * size.1 + self.1
    }

    fn from_usize_index(index: usize, size: TilePos) -> TilePos {
        TilePos::from((index / size.1, index % size.1))
    }

    fn size_to_max_index(&self) -> usize {
        self.0 * self.1 - 1
    }

    pub fn size_as_usize(&self) -> usize {
        self.0 * self.1
    }
}

impl Index<usize> for Board {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.content[index]
    }
}

impl IndexMut<usize> for Board {

    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.content[index]
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Control { Up, Down, Left, Right }

#[derive(Copy, Clone, Debug)]
pub enum Display {
    Create { pos: TilePos, value: u8 },
    CombineInto { a: TilePos, b: TilePos, target: TilePos },
    Move { from: TilePos, to: TilePos },
    ScoreAdd { add: usize },
    GameOver { score: usize }
}

// For value n as u8, the shown number is 2^(n-1). Specially, if n==0, there isn't a tile.
// That means, 0 => nothing, 1 => 2, 2 => 4, 3 => 8, ..., 11 => 2048.
#[derive(Debug, Eq, PartialEq)]
pub struct Board {
    content: Vec<u8>,
    size: TilePos,
    score: usize,
}

impl Board {
    pub fn new(size: impl Into<TilePos>) -> Board {
        let size = size.into();
        Board::from_raw_board(size, Vec::new())
    }

    fn from_raw_board(size: impl Into<TilePos>, mut content: Vec<u8>) -> Board {
        let size = size.into();
        content.resize(size.size_as_usize(), 0);
        Board {
            content,
            size,
            score: 0
        }
    }

    #[must_use]
    pub fn start_game(&mut self) -> Vec<Display> {
        gen(self, 4)
    }

    #[must_use]
    pub fn control_and_generate(&mut self, ctrl: Control) -> Vec<Display> {
        let mut vec = Vec::new();
        vec.append(&mut control_move(self, &ctrl).clone());
        vec.append(&mut gen(self, 2).clone());
        if is_game_over(self) {
            vec.push(Display::GameOver { score: self.score });
        };
        vec
    }
}

//////////////////////////////////// CONTROLLING ////////////////////////////////////

fn next_index(ct: &Control, ind: usize, &TilePos(rows, columns): &TilePos) -> Option<usize> {
    use Control::{Up, Down, Left, Right};
    match *ct {
        Up   => if ind >= columns * (rows - 1)  { None } else { Some(ind + columns) },
        Down => if ind < columns                { None } else { Some(ind - columns) },
        Left => if ind % columns == columns - 1 { None } else { Some(ind + 1) },
        Right => if ind % columns == 0          { None } else { Some(ind - 1) },
    }
}

fn each_start(ct: &Control, &TilePos(rows, columns): &TilePos) -> impl Iterator<Item = usize> {
    use Control::{Up, Down, Left, Right};
    let (begin, end, multiplier, minus) = match *ct {
        Up    => (0, columns, 1, 0),
        Down  => (columns * (rows - 1), columns * rows, 1, 0),
        Left  => (0, rows, columns, 0),
        Right => (1, rows + 1, columns, 1),
    };
    (begin .. end).map( move |k| k * multiplier).map(move |k| k - minus)
}

#[must_use]
fn control_move(board: &mut Board, ctrl: &Control) -> Vec<Display> {
    let mut ans = Vec::new();
    for start_ind in each_start(ctrl, &board.size) {
        let mut last_ind = start_ind;
        let mut ind = Vec::new();
        ind.push(last_ind);
        while let Some(next_ind) = next_index(ctrl, last_ind, &board.size) {
            ind.push(next_ind);
            last_ind = next_ind;
        }
        let target_ind = ind.clone();
        // 从左往右扫ab，忽略0，找到a!=b就将a取出，找到a==b就将ab合并
        // 先忽略0
        ind.retain(|&e| board[e] != 0);
        // 找！
        let mut ptr = 0;
        for i in 0..ind.len() {
            if board[ind[i]] == 0 {
                continue;
            }
            if i != ind.len() - 1 && board[ind[i]] == board[ind[i + 1]] {
                //合并i和i+1
                let val = board[ind[i]]; // in case that i or i+1 equals target_ind[ptr]
                board[ind[i]] = 0;
                board[ind[i + 1]] = 0;
                board[target_ind[ptr]] = val + 1;
                display_combine_into(&mut ans, ind[i + 1], ind[i], target_ind[ptr],&board);
                let add = 2usize << val as usize;
                ans.push(Display::ScoreAdd { add });
                board.score += add;
            } else if ind[i] != target_ind[ptr] { // filter unnecessary moves
                //取出i
                let val = board[ind[i]];
                board[ind[i]] = 0;
                board[target_ind[ptr]] = val;
                display_move(&mut ans, ind[i], target_ind[ptr], &board);
            }
            ptr += 1;
        }
    };
    ans
}

fn display_combine_into(v: &mut Vec<Display>, a: usize, b: usize, target: usize, bo: &Board) {
    let r = Display::CombineInto {
        a: TilePos::from_usize_index(a, bo.size),
        b: TilePos::from_usize_index(b, bo.size),
        target: TilePos::from_usize_index(target, bo.size)
    };
    v.push(r);
}

fn display_move(v: &mut Vec<Display>, f: usize, t: usize, b: &Board) {
    let r = Display::Move {
        from: TilePos::from_usize_index(f, b.size),
        to: TilePos::from_usize_index( t, b.size),
    };
    v.push(r);
}

//////////////////////////////////// GENERATE TILES ////////////////////////////////////

use rand::Rng;

fn gen_tile_value() -> u8 {
    if rand::thread_rng().gen_bool(0.9) { 1 } else { 2 }
}

fn gen_pos(board: &Board) -> Option<TilePos> {
    let mut available = Vec::new();
    for i in 0..=board.size.size_to_max_index() {
        if board[i] == 0 {
            available.push(TilePos::from_usize_index(i, board.size));
        }
    }
    rand::thread_rng().choose(&available).map(|a| *a)
}

#[must_use]
fn gen(board: &mut Board, count: usize) -> Vec<Display> {
    let size = board.size;
    let mut ans = Vec::new();
    for _ in 0..count {
        if let Some(pos) = gen_pos(&board) {
            let value = gen_tile_value();
            board[pos.to_usize_index(size)] = value;
            ans.push(Display::Create { pos, value })
        }
    }
    ans
}

//////////////////////////////////// GAME OVER LOGIC ////////////////////////////////////

fn is_game_over(board: &Board) -> bool {
    for i in 0..=board.size.size_to_max_index() {
        if board[i] == 0 {
            return false;
        }
        for ct in [Control::Right, Control::Down].iter() {
            if let Some(next) = next_index(ct, i, &board.size) {
                if board[i] == board[next] {
                    return false;
                }
            }
        }
    }
    true
}

//////////////////////////////////// UNIT TESTS ////////////////////////////////////

#[cfg(test)]
mod tests {
    use Board;
    use control_move;
    use Control;
    use TilePos;
    use is_game_over;
    use gen_tile_value;
    use gen_pos;
    use gen;
    use Display;

    #[test]
    fn index_usize_convert() {
        let cond = [
            ((300, 500), (0, 0), 0),
            ((300, 500), (0, 10), 10),
            ((300, 500), (0, 499), 499),
            ((300, 500), (1, 0), 500),
            ((300, 500), (1, 499), 999),
            ((300, 500), (299, 499), 300 * 500 - 1)
        ];
        for ((sx, sy), (x, y), i) in cond.iter() {
            let size = TilePos::from((*sx, *sy));
            let pos = TilePos::from((*x, *y));
            let i1 = pos.to_usize_index(size);
            assert_eq!(*i, i1);
            let i2 = TilePos::from_usize_index(i1, size);
            assert_eq!(pos, i2);
        }
    }

    #[test]
    fn index_new() {
        let cond = [
            ((10, 5), 10 * 5 - 1),
            ((300, 500), 300 * 500 - 1),
        ];
        for ((sx, sy), s) in cond.iter() {
            let size = TilePos::from((*sx, *sy));
            let i = size.size_to_max_index();
            assert_eq!(i, *s);
        }
    }

    #[test]
    fn board_new() {
        let g = Board::new((5, 10));
        assert_eq!(g.content.len(), 50);
        assert_eq!(g.size, TilePos::from((5, 10)));
    }

    #[test]
    fn control_move_output() {
        let mut g = Board::from_raw_board((2, 4), vec![
            1, 1, 0, 2,
            0, 4, 4, 2,
        ]);
        let a = control_move(&mut g, &Control::Left);
        let ans = String::from("[\
        CombineInto { a: TilePos(0, 1), b: TilePos(0, 0), target: TilePos(0, 0) }, \
        Move { from: TilePos(0, 3), to: TilePos(0, 1) }, \
        CombineInto { a: TilePos(1, 2), b: TilePos(1, 1), target: TilePos(1, 0) }, \
        Move { from: TilePos(1, 3), to: TilePos(1, 1) }]");
        assert_eq!(ans, format!("{:?}", a));
    }

    #[test]
    fn control_move_path() {
        let cond = [
            (Control::Left, Board::from_raw_board((8, 7), vec![
                2, 0, 0, 0, 0, 0, 0,
                7, 3, 0, 0, 0, 0, 0,
                3, 0, 0, 0, 0, 0, 0,
                4, 3, 5, 2, 1, 0, 0,
                3, 4, 3, 2, 0, 0, 0,
                3, 3, 2, 0, 0, 0, 0,
                2, 5, 2, 0, 0, 0, 0,
                2, 1, 2, 1, 2, 1, 2,
            ])),
            (Control::Right, Board::from_raw_board((8, 7), vec![
                0, 0, 0, 0, 0, 0, 2,
                0, 0, 0, 0, 0, 7, 3,
                0, 0, 0, 0, 0, 0, 3,
                0, 0, 4, 3, 5, 2, 1,
                0, 0, 0, 3, 4, 2, 3,
                0, 0, 0, 0, 2, 3, 3,
                0, 0, 0, 0, 2, 5, 2,
                2, 1, 2, 1, 2, 1, 2,
            ])),
            (Control::Up, Board::from_raw_board((8, 7), vec![
                3, 6, 6, 5, 3, 3, 3,
                2, 2, 4, 3, 3, 2, 2,
                0, 4, 2, 3, 2, 1, 0,
                0, 3, 4, 1, 0, 2, 0,
                0, 4, 2, 0, 0, 1, 0,
                0, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0,
            ])),
            (Control::Down, Board::from_raw_board((8, 7), vec![
                0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0,
                0, 6, 0, 0, 0, 0, 0,
                0, 2, 6, 0, 0, 2, 0,
                0, 4, 4, 5, 0, 3, 0,
                0, 3, 2, 3, 2, 1, 0,
                2, 4, 4, 3, 3, 2, 2,
                3, 1, 2, 1, 3, 1, 3
            ]))
        ];
        for (dir, target) in cond.iter() {
            let mut g = Board::from_raw_board((8, 7), vec![
                0, 0, 0, 0, 0, 2, 0,
                0, 6, 6, 0, 2, 2, 0,
                0, 2, 0, 0, 0, 2, 0,
                0, 4, 3, 5, 2, 1, 0,
                2, 2, 3, 3, 2, 2, 2,
                0, 2, 2, 2, 2, 0, 2,
                2, 4, 4, 2, 0, 0, 0,
                2, 1, 2, 1, 2, 1, 2,
            ]);
            let _ = control_move(&mut g, dir);
            assert_eq!(g, *target);
        }
    }

    #[test]
    fn board_game_over() {
        let cond = [
            (false, Board::from_raw_board((2, 2), vec![0, 0, 0, 0])),
            (false, Board::from_raw_board((2, 2), vec![1, 0, 0, 0])),
            (false, Board::from_raw_board((2, 2), vec![0, 1, 0, 0])),
            (false, Board::from_raw_board((2, 2), vec![0, 0, 1, 0])),
            (false, Board::from_raw_board((2, 2), vec![0, 0, 0, 1])),
            (false, Board::from_raw_board((2, 2), vec![
                0, 1,
                0, 1]
            )),
            (false, Board::from_raw_board((2, 2), vec![
                2, 2,
                0, 0]
            )),
            (false, Board::from_raw_board((2, 2), vec![
                1, 2,
                1, 3]
            )),
            (false, Board::from_raw_board((2, 2), vec![
                1, 2,
                0, 0]
            )),
            (true, Board::from_raw_board((2, 2), vec![1, 2, 3, 4])),
            (false, Board::from_raw_board((8, 7), vec![
                0, 0, 0, 0, 0, 2, 0,
                0, 6, 6, 0, 2, 2, 0,
                0, 2, 0, 0, 0, 2, 0,
                0, 4, 3, 5, 2, 1, 0,
                2, 2, 3, 3, 2, 2, 2,
                0, 2, 2, 2, 2, 0, 2,
                2, 4, 4, 2, 0, 0, 0,
                2, 1, 2, 1, 2, 1, 2,
            ]))
        ];
        for (res, b) in cond.iter() {
            let i = is_game_over(b);
            assert_eq!(*res, i);
        }
    }

    #[test]
    fn generate_tile_value() {
        for _ in 0..1000 {
            let a = gen_tile_value();
            assert!(a == 1 || a == 2);
        }
    }

    #[test]
    fn generate_pos() {
        let cond = [
            (Board::from_raw_board((2, 2), vec![
                0, 0,
                1, 0
            ]), Some(vec![(0, 0), (0, 1), (1, 1)])),
            (Board::from_raw_board((2, 3), vec![
                0, 3, 0,
                1, 2, 1
            ]), Some(vec![(0, 0), (0, 2)])),
            (Board::from_raw_board((2, 2), vec![
                2, 3,
                1, 4
            ]), None),
        ];
        fn check_contains(range: &Vec<(usize, usize)>, pos: TilePos) -> bool {
            let a = &(pos.0, pos.1);
            range.contains(a)
        }
        for (board, within) in cond.iter() {
            for _ in 0..1000 {
                let pos = gen_pos(board);
                match (within, pos) {
                    (None, Some(_)) => panic!("pos generated with no space left in board"),
                    (None, None) => continue,
                    (Some(_), None) => panic!("space in board not detected"),
                    (Some(range), Some(pos)) if !check_contains(range, pos) =>
                        panic!("pos generated outside bound"),
                    (Some(_), Some(_)) => continue,
                }
            }
        }
    }

    #[test]
    fn generate() {
        let mut board = Board::from_raw_board((2, 2), vec![
            0, 8,
            9, 0
        ]);
        let display = gen(&mut board, 2);
        for d in &display {
            if let Display::Create { pos, value } = d {
                assert!(vec![(0, 0), (1, 1)].contains(&(pos.0, pos.1)));
                assert!(vec![1, 2].contains(value));
            }
        }
    }
}

