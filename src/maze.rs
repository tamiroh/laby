pub const SIZE: usize = 21;
pub type Cell = (usize, usize);

pub struct Maze {
    walls: [[bool; SIZE]; SIZE],
    start: Cell,
    goal: Cell,
}

impl Maze {
    pub fn new(walls: [[bool; SIZE]; SIZE], start: Cell, goal: Cell) -> Self {
        let maze = Self { walls, start, goal };
        assert!(
            !maze.is_wall(start.0 as isize, start.1 as isize),
            "start must be a passage"
        );
        assert!(
            !maze.is_wall(goal.0 as isize, goal.1 as isize),
            "goal must be a passage"
        );
        maze
    }

    pub fn start(&self) -> Cell {
        self.start
    }

    pub fn goal(&self) -> Cell {
        self.goal
    }

    // Cells outside the maze are walls, including negative coordinates.
    pub fn is_wall(&self, x: isize, y: isize) -> bool {
        x < 0
            || y < 0
            || x >= SIZE as isize
            || y >= SIZE as isize
            || self.walls[y as usize][x as usize]
    }

    pub fn neighbors(&self, (x, y): Cell) -> impl Iterator<Item = Cell> + '_ {
        [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .filter_map(move |(dx, dy)| {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                (!self.is_wall(nx, ny)).then_some((nx as usize, ny as usize))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries_are_walls_and_neighbors_are_passages() {
        let mut walls = [[true; SIZE]; SIZE];
        walls[0][0] = false;
        walls[0][1] = false;
        let maze = Maze::new(walls, (0, 0), (1, 0));
        assert!(maze.is_wall(-1, 0));
        assert!(maze.is_wall(0, -1));
        assert!(maze.is_wall(SIZE as isize, 0));
        assert!(maze.is_wall(0, SIZE as isize));
        assert_eq!(maze.neighbors((0, 0)).collect::<Vec<_>>(), vec![(1, 0)]);
    }
}
