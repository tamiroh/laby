use std::f64::consts::{FRAC_PI_2, TAU};

pub const SIZE: usize = 21;

pub struct Game {
    pub walls: [[bool; SIZE]; SIZE],
    pub seen: [[bool; SIZE]; SIZE],
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub goal: (usize, usize),
    pub steps: usize,
    pub won: bool,
}

impl Game {
    pub fn new(mut seed: u64) -> Self {
        let mut walls = [[true; SIZE]; SIZE];
        let mut stack = vec![(1, 1)];
        walls[1][1] = false;
        while let Some(&(x, y)) = stack.last() {
            let neighbors: Vec<_> = [(0, -2), (2, 0), (0, 2), (-2, 0)]
                .into_iter()
                .filter_map(|(dx, dy)| {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    (nx > 0 && ny > 0 && nx < SIZE as isize - 1 && ny < SIZE as isize - 1)
                        .then_some((nx as usize, ny as usize))
                })
                .filter(|&(nx, ny)| walls[ny][nx])
                .collect();
            if neighbors.is_empty() {
                stack.pop();
                continue;
            }
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let (nx, ny) = neighbors[(seed >> 32) as usize % neighbors.len()];
            walls[(y + ny) / 2][(x + nx) / 2] = false;
            walls[ny][nx] = false;
            stack.push((nx, ny));
        }
        // Put the exit at the most distant reachable cell.
        let mut queue = std::collections::VecDeque::from([(1, 1)]);
        let mut visited = [[false; SIZE]; SIZE];
        visited[1][1] = true;
        let mut goal = (1, 1);
        while let Some((x, y)) = queue.pop_front() {
            goal = (x, y);
            for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                if !walls[ny][nx] && !visited[ny][nx] {
                    visited[ny][nx] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
        let mut game = Self {
            walls,
            seen: [[false; SIZE]; SIZE],
            x: 1.5,
            y: 1.5,
            angle: if !walls[1][2] { 0.0 } else { FRAC_PI_2 },
            goal,
            steps: 0,
            won: false,
        };
        game.reveal();
        game
    }

    pub fn wall(&self, x: f64, y: f64) -> bool {
        x < 0.0
            || y < 0.0
            || x >= SIZE as f64
            || y >= SIZE as f64
            || self.walls[y as usize][x as usize]
    }

    pub fn advance(&mut self, forward: f64, sideways: f64) {
        if self.won {
            return;
        }
        let dx = (self.angle.cos() * forward - self.angle.sin() * sideways) * 0.22;
        let dy = (self.angle.sin() * forward + self.angle.cos() * sideways) * 0.22;
        let old = (self.x, self.y);
        if self.clear(self.x + dx, self.y) {
            self.x += dx;
        }
        if self.clear(self.x, self.y + dy) {
            self.y += dy;
        }
        if old != (self.x, self.y) {
            self.steps += 1;
        }
        self.won = (self.x as usize, self.y as usize) == self.goal;
        self.reveal();
    }

    fn clear(&self, x: f64, y: f64) -> bool {
        [-0.18, 0.18].into_iter().all(|dx| {
            [-0.18, 0.18]
                .into_iter()
                .all(|dy| !self.wall(x + dx, y + dy))
        })
    }

    pub fn turn(&mut self, delta: f64) {
        if !self.won {
            self.angle = (self.angle + delta).rem_euclid(TAU);
        }
    }

    fn reveal(&mut self) {
        for y in 0..SIZE {
            for x in 0..SIZE {
                if (x as f64 + 0.5 - self.x).hypot(y as f64 + 0.5 - self.y) < 3.0 {
                    self.seen[y][x] = true;
                }
            }
        }
    }

    pub fn cast(&self, angle: f64) -> (f64, bool, f64) {
        let (dx, dy) = (angle.cos(), angle.sin());
        let (mut x, mut y) = (self.x.floor() as i32, self.y.floor() as i32);
        let (sx, sy) = (if dx < 0.0 { -1 } else { 1 }, if dy < 0.0 { -1 } else { 1 });
        let (delta_x, delta_y) = (1.0 / dx.abs(), 1.0 / dy.abs());
        let mut tx = if dx < 0.0 {
            self.x - x as f64
        } else {
            x as f64 + 1.0 - self.x
        } * delta_x;
        let mut ty = if dy < 0.0 {
            self.y - y as f64
        } else {
            y as f64 + 1.0 - self.y
        } * delta_y;
        loop {
            let (distance, side) = if tx < ty {
                x += sx;
                tx += delta_x;
                (tx - delta_x, false)
            } else {
                y += sy;
                ty += delta_y;
                (ty - delta_y, true)
            };
            if self.wall(x as f64, y as f64) {
                let texture = if side {
                    self.x + distance * dx
                } else {
                    self.y + distance * dy
                };
                return (distance.max(0.001), side, texture.fract());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_passage_and_exit_are_reachable() {
        for seed in 0..100 {
            let game = Game::new(seed);
            let mut visited = [[false; SIZE]; SIZE];
            let mut stack = vec![(1, 1)];
            while let Some((x, y)) = stack.pop() {
                if visited[y][x] {
                    continue;
                }
                visited[y][x] = true;
                for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                    if !game.walls[ny][nx] {
                        stack.push((nx, ny));
                    }
                }
            }
            assert!(visited[game.goal.1][game.goal.0]);
            assert_ne!(game.goal, (1, 1));
            for (y, row) in visited.iter().enumerate() {
                for (x, &reachable) in row.iter().enumerate() {
                    assert_eq!(reachable, !game.walls[y][x]);
                }
            }
        }
    }

    #[test]
    fn movement_cannot_cross_walls() {
        let mut game = Game::new(42);
        game.angle = std::f64::consts::PI;
        for _ in 0..100 {
            game.advance(1.0, 0.0);
        }
        assert!(game.x >= 1.18);
        assert!(!game.wall(game.x, game.y));
        assert!((game.cast(std::f64::consts::PI).0 - (game.x - 1.0)).abs() < 1e-9);
    }

    #[test]
    fn entering_exit_wins_and_stops_movement() {
        let mut game = Game::new(7);
        game.x = game.goal.0 as f64 + 0.5;
        game.y = game.goal.1 as f64 + 0.5;
        game.advance(0.0, 0.0);
        assert!(game.won);
        let position = (game.x, game.y);
        game.advance(1.0, 0.0);
        assert_eq!(position, (game.x, game.y));
    }
}
