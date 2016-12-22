use rand::{thread_rng, sample, Rng};
use std::char;
use std::iter;
use std::iter::FromIterator;
use std::string::ToString;

pub struct LatinSquare {
    size: u8,
    squares: Vec<u8>,
}

impl LatinSquare {
    pub fn random(size: u8) -> LatinSquare {
        let mut rng = thread_rng();

        let mut by_row = vec![vec![0; size as usize]; size as usize];
        for (y, row) in by_row.iter_mut().enumerate() {
            for (x, square) in row.iter_mut().enumerate() {
                *square = ((x + y) as u8) % size;
            }
        }
        rng.shuffle(&mut by_row);

        let mut by_col = vec![vec![0; size as usize]; size as usize];
        for (y, row) in by_row.into_iter().enumerate() {
            for (x, square) in row.into_iter().enumerate() {
                by_col[x][y] = square;
            }
        }
        rng.shuffle(&mut by_col);

        let mut ls = LatinSquare {
            size: size,
            squares: vec![!0; (size as usize).pow(2)],
        };

        for (x, col) in by_col.into_iter().enumerate() {
            for (y, square) in col.into_iter().enumerate() {
                *ls.square_mut(x as u8, y as u8) = square;
            }
        }

        ls
    }

    pub fn pprint(&self) -> String {
        let mut out = String::new();
        for y in 0..self.size {
            for x in 0..self.size {
                out.push(char::from_digit(self.square(x, y) as u32, 10).unwrap());
            }
            out += "\n";
        }
        out
    }

    fn static_square_coord(size: u8, x: u8, y: u8) -> usize {
        x as usize + y as usize * size as usize
    }

    fn square_coord(&self, x: u8, y: u8) -> usize {
        LatinSquare::static_square_coord(self.size, x, y)
    }

    fn square(&self, x: u8, y: u8) -> i8 {
        self.squares[self.square_coord(x, y)] as i8
    }

    fn square_mut(&mut self, x: u8, y: u8) -> &mut u8 {
        let coord = self.square_coord(x, y);
        &mut self.squares[coord]
    }
}

#[derive(Debug)]
pub struct UnequalLatinSquare {
    size: u8,
    squares: Vec<u32>,
    adjacency: Vec<Adjacency>,
    last_check_squares: Option<Vec<u32>>
}

impl UnequalLatinSquare {
    pub fn from_latin_square(ls: LatinSquare) -> UnequalLatinSquare {
        let size = ls.size as usize;
        let mut uls = UnequalLatinSquare {
            size: ls.size,
            squares: vec![0; size.pow(2)],
            adjacency: vec![Adjacency {above: false, below: false, left: false, right: false}; size.pow(2)],
            last_check_squares: None
        };

        for x in 0..ls.size {
            for y in 0..ls.size {
                let adj = Adjacency {
                    above: y > 0 && (ls.square(x, y) - ls.square(x, y - 1)).abs() == 1,
                    below: y < ls.size - 1 && (ls.square(x, y) - ls.square(x, y + 1)).abs() == 1,
                    left: x > 0 && (ls.square(x, y) - ls.square(x - 1, y)).abs() == 1,
                    right: x < ls.size - 1 && (ls.square(x, y) - ls.square(x + 1, y)).abs() == 1
                };
                if y > 0 {
                    assert_eq!(adj.above, uls.square_adjacents(x, y - 1).below);
                }
                if x > 0 {
                    assert_eq!(adj.left, uls.square_adjacents(x - 1, y).right);
                }
                *uls.square_adjacents_mut(x, y) = adj;
            }
        }

        let mut rng = thread_rng();
        let unscratched_amt = rng.gen_range(1, 6);
        let unscratched = sample(&mut rng, 0..uls.squares.len(), unscratched_amt);

        for (i, square) in ls.squares.iter().enumerate() {
            if unscratched.contains(&i) {
                uls.squares[i] = 1 << *square;
            } else {
                uls.squares[i] = (1 << ls.size) - 1;
            }
        }

        uls
    }

    fn tatham_puzzle_id_base(&self) -> String {
        let mut out = String::new();
        for y in 0..self.size {
            for x in 0..self.size {
                out += format!("{}", self.square_value(x, y).map_or(0, |v| v + 1)).as_str();
                let adj = self.square_adjacents(x, y);
                if adj.above {
                    out += "U";
                }
                if adj.below {
                    out += "D";
                }
                if adj.left {
                    out += "L";
                }
                if adj.right {
                    out += "R";
                }
                out += ",";
            }
        }
        out
    }

    pub fn tatham_puzzle_id(&self) -> String {
        format!("{}a:{}", self.size, self.tatham_puzzle_id_base())
    }

    pub fn to_tatham_save(&self) -> String {
        fn tatham_save_line<T: ToString>(name: &str, value: T) -> String {
            let v_str = value.to_string();
            format!("{: <8}:{}:{}\n", name, v_str.len(), v_str)
        }

        let mut out = String::new();
        out += "SAVEFILE:41:Simon Tatham's Portable Puzzle Collection\n";
        out += "VERSION :1:1\n";
        out += "GAME    :7:Unequal\n";
        out += tatham_save_line("PARAMS", self.size.to_string() + "adk").as_str();
        out += tatham_save_line("CPARAMS", self.size.to_string() + "adk").as_str();
        out += tatham_save_line("DESC", self.tatham_puzzle_id_base()).as_str();

        let mut moves = String::new();
        let mut move_count = 0;

        // No idea what this is for, but Tatham refuses to load it otherwise...
        for _ in 0..2 {
            moves += tatham_save_line("MOVE", "F1,1,2048").as_str();
            move_count += 1;
        }

        for x in 0..self.size {
            for y in 0..self.size {
                let poss_moves = self.square_possible(x, y);
                if poss_moves.len() > 1 {
                    for option in poss_moves {
                        moves += tatham_save_line("MOVE", format!("P{},{},{}", x, y, option + 1)).as_str();
                        move_count += 1;
                    }
                }
            }
        }

        out += tatham_save_line("NSTATES", move_count).as_str();
        out += tatham_save_line("STATEPOS", move_count).as_str();
        out += &moves;

        out
    }

    pub fn pprint(&self) -> String {
        let mut header = String::new();
        header.extend(iter::repeat('-').take(self.size as usize * 2 - 1));

        let mut out = String::new();
        out += "/";
        out += &header;
        out += "\\\n";
        for y in 0..self.size {
            out += "|";
            for x in 0..self.size {
                if let Some(value) = self.square_value(x, y) {
                    out.push(char::from_digit(value as u32 + 1, 10).unwrap());
                } else {
                    out.push('X');
                }
                if self.square_adjacents(x, y).right {
                    out.push('|');
                } else if x < self.size - 1 {
                    out.push(' ');
                }
            }
            out += "|\n";
            if y < self.size - 1 {
                out += "|";
                for x in 0..self.size {
                    if self.square_adjacents(x, y).below {
                        out += "-";
                    } else {
                        out += " ";
                    }
                    if x < self.size - 1 {
                        out += " ";
                    }
                }
                out += "|\n";
            }
        }
        out += "\\";
        out += &header;
        out += "/\n\n";

        for y in 0..self.size {
            for x in 0..self.size {
                out += format!("{},{} possible values: {:?}\n", x, y, Vec::from_iter(self.square_possible(x, y).into_iter().map(|v| v + 1))).as_str();
            }
            out += "\n";
        }

        out
    }

    fn square_coord(&self, x: u8, y: u8) -> usize {
        assert!(self.square_coord_is_legal(x as i8, y as i8));
        LatinSquare::static_square_coord(self.size, x, y)
    }

    fn square_coord_is_legal(&self, x: i8, y: i8) -> bool {
        x >= 0 && y >= 0 && x < self.size as i8 && y < self.size as i8
    }

    fn square_possible(&self, x: u8, y: u8) -> Vec<u8> {
        let mut i = 0;
        let mut mask = self.square_bitmask(x, y);
        let mut possible = Vec::new();

        while mask != 0 {
            if mask & 1 == 1 {
                possible.push(i);
            }

            i += 1;
            mask = mask.wrapping_shr(1);
        }

        possible
    }

    fn square_bitmask(&self, x: u8, y: u8) -> u32 {
        self.squares[self.square_coord(x, y)]
    }

    fn square_bitmask_mut(&mut self, x: u8, y: u8) -> &mut u32 {
        let coord = self.square_coord(x, y);
        &mut self.squares[coord]
    }

    fn square_value(&self, x: u8, y: u8) -> Option<u8> {
        let bitmask = self.square_bitmask(x, y);
        if bitmask.count_ones() != 1 {
            return None;
        }

        let mut i = 0;
        let mut testmask = 1;
        while testmask != bitmask {
            i += 1;
            testmask <<= 1;
        }
        Some(i)
    }

    fn square_adjacents(&self, x: u8, y: u8) -> Adjacency {
        self.adjacency[self.square_coord(x, y)]
    }

    fn square_adjacents_mut(&mut self, x: u8, y: u8) -> &mut Adjacency {
        let coord = self.square_coord(x, y);
        &mut self.adjacency[coord]
    }

    fn row_free(&self, p_x: u8, y: u8) -> u32 {
        let mut taken = 0;
        for x in 0..self.size {
            let square = self.square_bitmask(x, y);
            if square.count_ones() == 1 && x != p_x {
                taken |= square;
            }
        }
        !taken
    }

    fn col_free(&self, x: u8, p_y: u8) -> u32 {
        let mut taken = 0;
        for y in 0..self.size {
            let square = self.square_bitmask(x, y);
            if square.count_ones() == 1 && y != p_y {
                taken |= square;
            }
        }
        !taken
    }

    pub fn solved(&self) -> bool {
        for (i, mask) in self.squares.iter().enumerate() {
            if mask.count_ones() == 0 {
                println!("Square {} has no possible values!", i);
            }

            if mask.count_ones() > 1 {
                return false;
            }
        }
        true
    }

    pub fn solve_step(&mut self) {
        self.mark_step();

        for x in 0..self.size {
            for y in 0..self.size {
                let mut mask = self.square_bitmask(x, y);
                mask &= self.row_free(x, y);
                mask &= self.col_free(x, y);
                for dir in Direction::all() {
                    if let Some((dir_x, dir_y)) = dir.square_coords(&self, x, y) {
                        let dir_square = self.square_bitmask(dir_x, dir_y);
                        if self.square_adjacents(x, y).in_direction(dir) {
                            mask &= dir_square.wrapping_shl(1) | dir_square.wrapping_shr(1);
                        } else {
                            // mask &= !(dir_square.wrapping_shl(1) & dir_square.wrapping_shr(1))
                            let mut dir_allowed = 0;
                            for digit in 0..self.size {
                                let digit_mask = 1 << digit;
                                if dir_square & digit_mask != 0 {
                                    dir_allowed |= !(digit_mask.wrapping_shl(1) | digit_mask | digit_mask.wrapping_shr(1));
                                }
                            }
                            mask &= dir_allowed;
                        }
                    }
                }
                *self.square_bitmask_mut(x, y) = mask;
            }
        }
    }

    fn mark_step(&mut self) {
        assert_ne!(Some(&self.squares), self.last_check_squares.as_ref());
        self.last_check_squares = Some(self.squares.clone());
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Above,
    Below,
    Left,
    Right
}

impl Direction {
    fn all() -> Vec<Direction> {
        vec![Direction::Above, Direction::Below, Direction::Left, Direction::Right]
    }

    fn square_coords(self, uls: &UnequalLatinSquare, prev_x: u8, prev_y: u8) -> Option<(u8, u8)> {
        let (x, y) = match self {
            Direction::Above => (prev_x as i8, prev_y as i8 - 1),
            Direction::Below => (prev_x as i8, prev_y as i8 + 1),
            Direction::Left => (prev_x as i8 - 1, prev_y as i8),
            Direction::Right => (prev_x as i8 + 1, prev_y as i8)
        };
        if uls.square_coord_is_legal(x, y) {
            Some((x as u8, y as u8))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Adjacency {
    above: bool,
    below: bool,
    left: bool,
    right: bool
}

impl Adjacency {
    fn in_direction(&self, dir: Direction) -> bool {
        match dir {
            Direction::Above => self.above,
            Direction::Below => self.below,
            Direction::Left => self.left,
            Direction::Right => self.right
        }
    }
}
