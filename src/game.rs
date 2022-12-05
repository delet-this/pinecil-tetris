use bitvec::{bitarr, BitArr, order::Lsb0};
use oorandom::Rand32;

enum MoveDirection {
    Left,
    Right,
}

#[derive(Clone)]
pub struct Block {
    pub shape: [BitArr!(for 4); 4], 
    pub size: u8,
    pub pos: (i8, i8), 
}

impl Block {
    pub fn move_left(&mut self) {
        self.pos.0 -= 1;
    }

    pub fn move_right(&mut self) {
        self.pos.0 += 1;
    }

    pub fn create_square() -> Self {
        Self { 
            shape: [
                bitarr![1, 1, 0, 0],
                bitarr![1, 1, 0, 0],
                bitarr![0, 0, 0, 0],
                bitarr![0, 0, 0, 0],
            ],
            size: 2,
            pos: (8/2, 1),
        }
    }

    pub fn create_l() -> Self {
        Self { 
            shape: [
                bitarr![0, 0, 1, 0],
                bitarr![1, 1, 1, 0],
                bitarr![0, 0, 0, 0],
                bitarr![0, 0, 0, 0],
            ],
            size: 3,
            pos: (8/2, 1),
        }
    }

    pub fn create_j() -> Self {
        Self { 
            shape: [
                bitarr![1, 0, 0, 0],
                bitarr![1, 1, 1, 0],
                bitarr![0, 0, 0, 0],
                bitarr![0, 0, 0, 0],
            ],
            size: 3,
            pos: (8/2, 1),
        }
    }

    pub fn create_z() -> Self {
        Self { 
            shape: [
                bitarr![1, 1, 0, 0],
                bitarr![0, 1, 1, 0],
                bitarr![0, 0, 0, 0],
                bitarr![0, 0, 0, 0],
            ],
            size: 3,
            pos: (8/2, 1),
        }
    }

    pub fn create_s() -> Self {
        Self { 
            shape: [
                bitarr![0, 1, 1, 0],
                bitarr![1, 1, 0, 0],
                bitarr![0, 0, 0, 0],
                bitarr![0, 0, 0, 0],
            ],
            size: 3,
            pos: (8/2, 1),
        }
    }

    pub fn create_t() -> Self {
        Self { 
            shape: [
                bitarr![0, 1, 0, 0],
                bitarr![1, 1, 1, 0],
                bitarr![0, 0, 0, 0],
                bitarr![0, 0, 0, 0],
            ], 
            size: 3,
            pos: (8/2, 1)
        }
    }

    pub fn create_i() -> Self {
        Self { 
            shape: [
                bitarr![0, 0, 0, 0],
                bitarr![1, 1, 1, 1],
                bitarr![0, 0, 0, 0],
                bitarr![0, 0, 0, 0],
            ], 
            size: 4,
            pos: (8/2, 2)
        }
    }
}

pub struct Tetris {
    current_block: Option<Block>,
    block_cooldown: u8,
    grid: [BitArr!(for 8); 32],
    rng: Rand32,
    move_direction: MoveDirection,
    score: u32,
    has_ended: bool,
}

impl Tetris {
    pub fn init() -> Self {
        Self {
            current_block: None,
            block_cooldown: 0,
            grid: [bitarr![0; 8]; 32],
            rng: Rand32::new(8),
            move_direction: MoveDirection::Left,
            score: 0,
            has_ended: false,
        }
    }

    pub fn add_block(&mut self) {
        if self.current_block.is_none() {
            let block = match self.rng.rand_range(0..7) {
                0 => Block::create_square(),
                1 => Block::create_l(),
                2 => Block::create_j(),
                3 => Block::create_z(),
                4 => Block::create_s(),
                5 => Block::create_t(),
                _ => Block::create_i(),
            };
            self.current_block = Some(block);
        }
    }

    pub fn rotate_block(&mut self) {
        if let Some(block) = self.current_block.clone() {
            let mut rotated_block: Block = block.clone();
            // the bitarray is rectangular
            let dim: usize = block.size.into();
            for i in 0..dim {
                for j in 0..dim {
                    rotated_block.shape[j].set(dim - 1 - i, block.shape[i][j]);
                }
            }
            if self.bounds_check(&rotated_block) {
                self.current_block.replace(rotated_block);
            }
        }
    }

    fn bounds_check(&mut self, block: &Block) -> bool {
        let dim = block.shape.len();
        for i in 0..dim {
            for j in 0..dim {
                if let Some(bit) = block.shape[i].get(j) {
                    let x = block.pos.0 + j as i8;
                    let y = block.pos.1 + i as i8;
                    if bit == true {
                        if y < 0 || y > (self.grid.len()-1) as i8 || x > 7 || x < 0 {
                            return false;
                        } else if let Some(grid_bit) = self.grid[y as usize].get(x as usize) {
                            if grid_bit == true {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    }

    pub const fn get_grid(&self) -> [BitArr!(for 8); 32] {
        self.grid
    }

    pub fn get_block(&self) -> Option<Block> {
        self.current_block.clone()
    }

    pub const fn get_score(&self) -> u32 {
        self.score
    }

    fn reached_bottom(&self) -> bool {
        if let Some(block) = &self.current_block {
            for (i, _) in block.shape.iter().enumerate() {
                for (j, bit) in block.shape[i].iter().enumerate() {
                    let x = block.pos.0 as usize + j;
                    let y = block.pos.1 as usize + i;
                    if y >= (self.grid.len()-1) {
                        if *bit {
                            return true;
                        }
                        continue;
                    }
                    else if let Some(grid_bit) = self.grid[y+1].get(x) {
                        if *bit && *grid_bit {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn fall(&mut self) {
        if let Some(block) = &mut self.current_block {
            let mut fallen_block: Block = block.clone();
            fallen_block.pos.1 += 1;
            // if self.bounds_check(&fallen_block) {
                self.current_block.replace(fallen_block);
            // }
        }
    }

    fn block_to_grid(&mut self) {
        if let Some(block) = &self.current_block {
            for (i, _) in block.shape.iter().enumerate() {
                for (j, bit) in block.shape[i].iter().enumerate() {
                    if *bit {
                        let x = j + block.pos.0 as usize;
                        let y = i + block.pos.1 as usize;
                        self.grid[y].set(x, true);
                    }
                }
            }
        }
        self.current_block = None;
        self.block_cooldown = 5;
    }

    fn clear_line(&mut self, row: usize) {
        // iterate rows above cleared row
        for y in (0..row).rev() {
            for x in 0..self.grid[y].len() {
                let mut b: Option<bool> = None;
                if let Some(bit) = self.grid[y].get(x) { b = Some(*bit); }
                // copy value to row below
                if let Some(bb) = b {
                    self.grid[y+1].set(x, bb);
                }
                self.grid[y].set(x, false);
            }
        }
    }

    fn check_line_clears(&mut self) {
        let mut is_clear = true;
        while is_clear {
            is_clear = false;
            let mut clear_row_y = 0;
            for (y, row) in self.grid.iter().enumerate() {
                // check if row is clear
                if row.count_ones() >= 7 {
                    is_clear = true;
                    clear_row_y = y;
                    break;
                }
            }
            if is_clear {
                self.clear_line(clear_row_y);
                self.score += 1;
            }
        }
    }

    fn clipping_top(&self) -> bool {
        if let Some(block) = &self.current_block {
            for (i, _) in block.shape.iter().enumerate() {
                for (_, bit) in block.shape[i].iter().enumerate() {
                    let y = block.pos.1 + i as i8;
                    if y <= 1 && *bit {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn block_at_edge(&self) -> bool {
        if let Some(block) = &self.current_block {
            for (i, _) in block.shape.iter().enumerate() {
                for (j, bit) in block.shape[i].iter().enumerate() {
                    if *bit {
                        let x = j as i8 + block.pos.0;
                        if x == 0 || x == 7 {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    pub fn move_block(&mut self) {
        if let Some(block) = self.current_block.clone() {
            let mut moved_block = block;
            match &self.move_direction {
                MoveDirection::Left => {
                    moved_block.move_left();
                    if !self.bounds_check(&moved_block) {
                        moved_block.move_right();
                        moved_block.move_right();
                        self.move_direction = MoveDirection::Right;
                    }                
                }
                MoveDirection::Right => {
                    moved_block.move_right();
                    if !self.bounds_check(&moved_block) {
                        moved_block.move_left();
                        moved_block.move_left();
                        self.move_direction = MoveDirection::Left;
                    }                
                }
            }
            self.current_block.replace(moved_block);
        }
    }

    pub const fn has_ended(&self) -> bool {
        self.has_ended
    }

    pub fn run(&mut self) {
        if self.has_ended {
            return;
        }
        if self.current_block.is_some() {
            if self.reached_bottom() {
                if self.clipping_top() {
                    self.has_ended = true;
                }
                self.block_to_grid();
                self.check_line_clears();
            } else {
                self.fall();
            }
        } else if self.block_cooldown > 0 {
            self.block_cooldown -= 1;
        } else {
            self.add_block();
        }
    }
}
