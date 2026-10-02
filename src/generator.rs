use crate::maze::{Maze, SIZE};

pub fn generate(mut seed: u64) -> Maze {
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
    let maze = Maze::new(walls, (1, 1), (1, 1));
    // Put the exit at the most distant reachable cell.
    let mut queue = std::collections::VecDeque::from([(1, 1)]);
    let mut visited = [[false; SIZE]; SIZE];
    visited[1][1] = true;
    let mut goal = (1, 1);
    while let Some((x, y)) = queue.pop_front() {
        goal = (x, y);
        for (nx, ny) in maze.neighbors((x, y)) {
            if !visited[ny][nx] {
                visited[ny][nx] = true;
                queue.push_back((nx, ny));
            }
        }
    }
    Maze::new(walls, (1, 1), goal)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_passage_and_exit_are_reachable() {
        for seed in 0..100 {
            let maze = generate(seed);
            let mut visited = [[false; SIZE]; SIZE];
            let mut stack = vec![maze.start()];
            while let Some((x, y)) = stack.pop() {
                if visited[y][x] {
                    continue;
                }
                visited[y][x] = true;
                stack.extend(maze.neighbors((x, y)));
            }
            assert!(visited[maze.goal().1][maze.goal().0]);
            assert_ne!(maze.goal(), (1, 1));
            for (y, row) in visited.iter().enumerate() {
                for (x, &reachable) in row.iter().enumerate() {
                    assert_eq!(reachable, !maze.is_wall(x as isize, y as isize));
                }
            }
        }
    }
}
