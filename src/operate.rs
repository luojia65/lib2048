use Direction;
use Game;

pub trait Operate {
    fn go(&mut self, dir: Direction);
}

impl Operate for Game {
    fn go(&mut self, dir: Direction) {
        stack_to_one_side(self, &dir);
        combine_to_higher(self, &dir);
        stack_to_one_side(self, &dir);
    }
}

/*
    rows: 3, columns: 4             rows: 3, columns: 5
    | 0  | 1  | 2  | 3  |           | 0  | 1  | 2  | 3  | 4  |
    | 4  | 5  | 6  | 7  |           | 5  | 6  | 7  | 8  | 9  |
    | 8  | 9  | 10 | 11 |           | 10 | 11 | 12 | 13 | 14 |
*/
fn stack_to_one_side(g: &mut Game, dir: &Direction){
    for start in dir.each_start(g.rows, g.columns) {
        let mut v = Vec::new();
        let mut last_ind = start;
        if g.board[start] != 0 {
            v.push(g.board[start]);
        }
        while let Some(ind) = dir.next_index(last_ind, g.rows, g.columns) {
            if g.board[ind] != 0 {
                v.push(g.board[ind]);
            }
            last_ind = ind;
        }
        v.reverse();
        let mut last_ind = start;
        while let Some(ind) = dir.next_index(last_ind, g.rows, g.columns) {
            g.board[last_ind] = if let Some(val) = v.pop() { val } else { 0 };
            last_ind = ind;
        }
        g.board[last_ind] = if let Some(val) = v.pop() { val } else { 0 };
    }
}
// 2*2 -> 3, 3*3 -> 4, etc
// returns if the game is modified
fn combine_to_higher(g: &mut Game, dir: &Direction) {
    for start in dir.each_start(g.rows, g.columns) {
        let mut last_ind = start;
        while let Some(ind) = dir.next_index(last_ind, g.rows, g.columns) {
            // Direction::Left,     start, ..., last_ind, ind, ...
            if g.board[ind] == g.board[last_ind] && g.board[ind] != 0 {
                g.board[last_ind] += 1;
                g.board[ind] = 0;
            }
            last_ind = ind;
        }
    }
}

#[cfg(test)]
mod tests {
    use game::Game;
    use direction::Direction;
    use operate::{Operate, stack_to_one_side, combine_to_higher};

    #[test]
    fn test_next_index() {
        use self::Direction::{Up, Down, Left, Right};
        let cond = [
            ((Down,   0, 3, 4), None),      ((Up,  0, 3, 4), Some(4)),
            ((Right, 0, 3, 4), None),      ((Left, 0, 3, 4), Some(1)),
            ((Down,   3, 3, 4), None),      ((Up,  3, 3, 4), Some(7)),
            ((Right, 3, 3, 4), Some(2)),   ((Left, 3, 3, 4), None),
            ((Down,   8, 3, 4), Some(4)),   ((Up,  8, 3, 4), None),
            ((Right, 8, 3, 4), None),      ((Left, 8, 3, 4), Some(9)),
            ((Down,   11, 3, 4), Some(7)),  ((Up,  11, 3, 4), None),
            ((Right, 11, 3, 4), Some(10)), ((Left, 11, 3, 4), None),
            ((Down,   6, 3, 5), Some(1)),   ((Up,  6, 3, 5), Some(11)),
            ((Right, 6, 3, 5), Some(5)),   ((Left, 6, 3, 5), Some(7)),
        ];
        for ((dir, ind, rows, columns), result) in cond.iter() {
            let next_index = dir.next_index(*ind, *rows, *columns);
            assert_eq!(next_index, *result);
        }
    }
    #[test]
    fn test_each_start() {
        use self::Direction::{Up, Down, Left, Right};
        let cond = [
            ((Down,   3, 4), vec![8, 9, 10, 11]),       ((Up, 3, 4),  vec![0, 1, 2, 3]),
            ((Right, 3, 4), vec![3, 7, 11]),           ((Left, 3, 4), vec![0, 4, 8]),
            ((Down,   3, 5), vec![10, 11, 12, 13, 14]), ((Up, 3, 5),  vec![0, 1, 2, 3, 4]),
            ((Right, 3, 5), vec![4, 9, 14]),           ((Left, 3, 5), vec![0, 5, 10]),
        ];
        for ((dir, rows, columns), result) in cond.iter() {
            let vec: Vec<usize> = dir.each_start(*rows, *columns).collect();
            assert_eq!(vec, *result);
        }
    }
    #[test]
    fn test_stack_to_one_side() {
        fn test_one(v: Vec<u8>, d: Direction) {
            let mut g = Game::from_raw_board(vec![
                0, 0, 0, 0,
                0, 0, 0, 2,
                0, 0, 2, 0,
                0, 2, 0, 0,
                2, 0, 0, 0,
                2, 0, 3, 4,
                2, 2, 3, 3,
                1, 2, 3, 4,
            ], 8, 4);
            let g1 = Game::from_raw_board(v, 8, 4);
            stack_to_one_side(&mut g, &d);
            assert_eq!(g, g1);
        };
        test_one(vec![
            0, 0, 0, 0,
            2, 0, 0, 0,
            2, 0, 0, 0,
            2, 0, 0, 0,
            2, 0, 0, 0,
            2, 3, 4, 0,
            2, 2, 3, 3,
            1, 2, 3, 4,
        ], Direction::Left);
        test_one(vec![
            0, 0, 0, 0,
            0, 0, 0, 2,
            0, 0, 0, 2,
            0, 0, 0, 2,
            0, 0, 0, 2,
            0, 2, 3, 4,
            2, 2, 3, 3,
            1, 2, 3, 4,
        ], Direction::Right);
        test_one(vec![
            2, 2, 2, 2,
            2, 2, 3, 4,
            2, 2, 3, 3,
            1, 0, 3, 4,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ], Direction::Up);
        test_one(vec![
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            2, 0, 2, 2,
            2, 2, 3, 4,
            2, 2, 3, 3,
            1, 2, 3, 4,
        ], Direction::Down);
    }

    #[test]
    fn test_combine_to_higher() {
        let mut g = Game::from_raw_board(vec![
            2, 2, 3, 3, 2, 2, 2,
            2, 2, 2, 2, 2, 0, 0,
            2, 4, 4, 2, 0, 0, 0,
            2, 6, 4, 5, 3, 1, 7,
        ], 4, 7);
        let g1 = Game::from_raw_board(vec![
            3, 0, 4, 0, 3, 0, 2,
            3, 0, 3, 0, 2, 0, 0,
            2, 5, 0, 2, 0, 0, 0,
            2, 6, 4, 5, 3, 1, 7,
        ], 4, 7);
        combine_to_higher(&mut g, &Direction::Left);
        assert_eq!(g, g1);
    }

    #[test]
    fn test_operate() {
        let mut g = Game::from_raw_board(vec![
            2, 2, 3, 3, 2, 2, 2,
            2, 2, 2, 2, 2, 0, 0,
            2, 4, 4, 2, 0, 0, 0,
            2, 6, 4, 5, 3, 1, 7,
            0, 0, 0, 0, 0, 2, 2,
        ], 5, 7);
        let g1 = Game::from_raw_board(vec![
            3, 4, 3, 2, 0, 0, 0,
            3, 3, 2, 0, 0, 0, 0,
            2, 5, 2, 0, 0, 0, 0,
            2, 6, 4, 5, 3, 1, 7,
            3, 0, 0, 0, 0, 0, 0,
        ], 5, 7);
        g.go(Direction::Left);
        assert_eq!(g, g1);
    }
}

