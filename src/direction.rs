pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    pub(crate) fn next_index(&self, ind: usize, rows: usize, columns: usize) -> Option<usize> {
        use self::Direction::{Up, Down, Left, Right};
        match self {
            Up   => if ind >= columns * (rows - 1)  { None } else { Some(ind + columns) },
            Down => if ind < columns                { None } else { Some(ind - columns) },
            Left => if ind % columns == columns - 1 { None } else { Some(ind + 1) },
            Right => if ind % columns == 0           { None } else { Some(ind - 1) },
        }
    }

    pub(crate) fn each_start(&self, rows: usize, columns: usize) -> impl Iterator<Item = usize> {
        use self::Direction::{Up, Down, Left, Right};
        let (begin, end, multiplier, minus) = match self {
            Up    => (0, columns, 1, 0),
            Down  => (columns * (rows - 1), columns * rows, 1, 0),
            Left  => (0, rows, columns, 0),
            Right => (1, rows + 1, columns, 1),
        };
        (begin .. end).map( move |k| k * multiplier).map(move |k| k - minus)
    }
}
