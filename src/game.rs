use std::{
    f64::consts::{FRAC_PI_2, TAU},
    time::Duration,
};

pub const SIZE: usize = 21;

#[derive(Clone, Copy)]
pub enum Action {
    Forward,
    Backward,
    Left,
    Right,
}

struct Motion {
    from: (f64, f64, f64),
    to: (f64, f64, f64),
    elapsed: Duration,
    moving: bool,
}

const MOTION_DURATION: Duration = Duration::from_millis(180);

pub struct Game {
    pub walls: [[bool; SIZE]; SIZE],
    pub seen: [[bool; SIZE]; SIZE],
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub goal: (usize, usize),
    pub steps: usize,
    pub won: bool,
    motion: Option<Motion>,
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
            motion: None,
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

    pub fn act(&mut self, action: Action) {
        // Finish each step before accepting another, so key repeat cannot build a backlog.
        if self.won || self.motion.is_some() {
            return;
        }
        let mut to = (self.x, self.y, self.angle);
        let moving = matches!(action, Action::Forward | Action::Backward);
        match action {
            Action::Forward | Action::Backward => {
                let direction = if matches!(action, Action::Forward) {
                    1.0
                } else {
                    -1.0
                };
                to.0 += self.angle.cos().round() * direction;
                to.1 += self.angle.sin().round() * direction;
                if self.wall(to.0, to.1) {
                    return;
                }
            }
            Action::Left => to.2 -= FRAC_PI_2,
            Action::Right => to.2 += FRAC_PI_2,
        }
        self.motion = Some(Motion {
            from: (self.x, self.y, self.angle),
            to,
            elapsed: Duration::ZERO,
            moving,
        });
    }

    pub fn update(&mut self, elapsed: Duration) {
        let Some(motion) = &mut self.motion else {
            return;
        };
        motion.elapsed += elapsed;
        let progress = (motion.elapsed.as_secs_f64() / MOTION_DURATION.as_secs_f64()).min(1.0);
        let eased = progress * progress * (3.0 - 2.0 * progress);
        self.x = motion.from.0 + (motion.to.0 - motion.from.0) * eased;
        self.y = motion.from.1 + (motion.to.1 - motion.from.1) * eased;
        self.angle = (motion.from.2 + (motion.to.2 - motion.from.2) * eased).rem_euclid(TAU);
        if progress >= 1.0 {
            self.x = motion.to.0;
            self.y = motion.to.1;
            self.angle = motion.to.2.rem_euclid(TAU);
            self.steps += usize::from(motion.moving);
            self.motion = None;
            self.won = (self.x as usize, self.y as usize) == self.goal;
            self.reveal();
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
            game.act(Action::Forward);
            game.update(MOTION_DURATION);
        }
        assert_eq!((game.x, game.y), (1.5, 1.5));
        assert_eq!(game.steps, 0);
        assert!((game.cast(std::f64::consts::PI).0 - 0.5).abs() < 1e-9);
    }

    #[test]
    fn steps_animate_between_cell_centers_and_can_reverse() {
        let mut game = Game::new(42);
        let start = (game.x, game.y);
        let target = (
            game.x + game.angle.cos().round(),
            game.y + game.angle.sin().round(),
        );
        game.act(Action::Forward);
        assert_eq!((game.x, game.y), start);
        game.update(MOTION_DURATION / 2);
        assert_eq!(
            (game.x, game.y),
            ((start.0 + target.0) / 2.0, (start.1 + target.1) / 2.0)
        );
        assert_eq!(game.steps, 0);
        // Inputs during animation must not interrupt or queue up extra movement.
        game.act(Action::Right);
        game.act(Action::Forward);
        game.update(MOTION_DURATION / 2);
        assert_eq!((game.x, game.y), target);
        assert_eq!(game.steps, 1);
        game.act(Action::Backward);
        game.update(Duration::from_secs(1));
        assert_eq!((game.x, game.y), start);
        assert_eq!(game.steps, 2);
    }

    #[test]
    fn turns_animate_ninety_degrees_across_angle_wrap() {
        let mut game = Game::new(42);
        game.angle = 0.0;
        game.act(Action::Left);
        game.update(MOTION_DURATION / 2);
        assert!((game.angle - 7.0 * std::f64::consts::FRAC_PI_4).abs() < 1e-9);
        game.update(MOTION_DURATION / 2);
        assert!((game.angle - 3.0 * FRAC_PI_2).abs() < 1e-9);
        game.act(Action::Right);
        game.update(MOTION_DURATION);
        assert!(game.angle.abs() < 1e-9);
        for _ in 0..4 {
            game.act(Action::Right);
            game.update(MOTION_DURATION);
        }
        assert!(game.angle.abs() < 1e-9);
        assert_eq!((game.x, game.y), (1.5, 1.5));
        assert_eq!(game.steps, 0);
    }

    #[test]
    fn entering_exit_wins_only_after_animation_and_stops_movement() {
        let mut game = Game::new(7);
        // Make the first reachable cell the exit to exercise a complete arrival.
        game.goal = (
            (game.x + game.angle.cos().round()) as usize,
            (game.y + game.angle.sin().round()) as usize,
        );
        game.act(Action::Forward);
        game.update(MOTION_DURATION / 2);
        assert!(!game.won);
        game.update(MOTION_DURATION / 2);
        assert!(game.won);
        let position = (game.x, game.y, game.angle);
        game.act(Action::Backward);
        game.act(Action::Left);
        game.update(MOTION_DURATION);
        assert_eq!(position, (game.x, game.y, game.angle));
    }
}
