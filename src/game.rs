// Runtime game
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Game {
    // 0 -> nothing, 2 -> 2, 3 -> 4, 4 -> 8, 5 -> 16 ... 11 -> 2048
    pub(crate) board: Vec<u8>,
    pub(crate) rows: usize,
    pub(crate) columns:usize
}

impl Game {
    pub fn new(rows: usize, columns: usize) -> Game {
        Game::from_raw_board(Vec::with_capacity(rows * columns), rows, columns)
    }

    pub(crate) fn from_raw_board(board: Vec<u8>, rows: usize, columns: usize) -> Game {
        if rows == 0 || columns == 0 {
            panic!("rows or columns equals zero; it's an invalid game board.");
        }
        Game {
            board,
            rows,
            columns
        }
    }
}

