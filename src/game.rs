use std::{
    f64::consts::{FRAC_PI_2, TAU},
    time::Duration,
};

use crate::maze::{Maze, SIZE};

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
    pub maze: Maze,
    pub seen: [[bool; SIZE]; SIZE],
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub steps: usize,
    pub won: bool,
    motion: Option<Motion>,
}

impl Game {
    pub fn new(maze: Maze) -> Self {
        let (x, y) = maze.start();
        let angle = maze.neighbors((x, y)).next().map_or(0.0, |(nx, ny)| {
            (ny as f64 - y as f64)
                .atan2(nx as f64 - x as f64)
                .rem_euclid(TAU)
        });
        let mut game = Self {
            seen: [[false; SIZE]; SIZE],
            x: x as f64 + 0.5,
            y: y as f64 + 0.5,
            angle,
            maze,
            steps: 0,
            won: false,
            motion: None,
        };
        game.reveal();
        game
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
                if self
                    .maze
                    .is_wall(to.0.floor() as isize, to.1.floor() as isize)
                {
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
            self.won = (self.x as usize, self.y as usize) == self.maze.goal();
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
            if self.maze.is_wall(x as isize, y as isize) {
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

    fn corridor(goal: (usize, usize)) -> Game {
        let mut walls = [[true; SIZE]; SIZE];
        walls[1][1..=3].fill(false);
        Game::new(Maze::new(walls, (1, 1), goal))
    }

    #[test]
    fn movement_cannot_cross_walls() {
        let mut game = corridor((3, 1));
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
        let mut game = corridor((3, 1));
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
        let mut game = corridor((3, 1));
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
        let mut game = corridor((2, 1));
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
