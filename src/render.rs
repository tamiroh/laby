use crate::{game::Game, maze::SIZE};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};
use std::{
    io::{self, Write},
    time::Duration,
};

#[derive(Clone, Copy)]
struct Cell {
    glyph: char,
    color: Color,
}
const SKY: Color = Color::Rgb {
    r: 13,
    g: 19,
    b: 31,
};
const GOLD: Color = Color::Rgb {
    r: 238,
    g: 190,
    b: 104,
};
const GREEN: Color = Color::Rgb {
    r: 100,
    g: 245,
    b: 164,
};

fn text(cells: &mut [Cell], width: usize, x: usize, y: usize, value: &str, color: Color) {
    for (offset, glyph) in value.chars().take(width.saturating_sub(x)).enumerate() {
        if let Some(cell) = cells.get_mut(y * width + x + offset) {
            *cell = Cell { glyph, color };
        }
    }
}

pub fn draw(
    out: &mut impl Write,
    game: &Game,
    width: u16,
    height: u16,
    map: bool,
    elapsed: Duration,
) -> io::Result<()> {
    // Leave the last column unused to avoid terminal auto-wrap.
    let w = width.saturating_sub(1) as usize;
    let h = height as usize;
    if w == 0 || h == 0 {
        return Ok(());
    }
    let mut cells = vec![
        Cell {
            glyph: ' ',
            color: Color::White
        };
        w * h
    ];
    if w < 58 || h < 20 {
        text(
            &mut cells,
            w,
            0,
            0,
            "Resize terminal to at least 59 x 20. Esc: quit",
            GOLD,
        );
    } else {
        let view_h = h - 5;
        let fov = 1.1_f64;
        let projection = w as f64 / (2.0 * (fov / 2.0).tan()) * 0.48;
        let mut depth = vec![0.0; w];
        for (x, depth_column) in depth.iter_mut().enumerate() {
            let offset = (((x as f64 + 0.5) / w as f64 * 2.0 - 1.0) * (fov / 2.0).tan()).atan();
            let (distance, side, texture) = game.cast(game.angle + offset);
            *depth_column = distance * offset.cos();
            let wall_h = projection / *depth_column;
            for y in 0..view_h {
                let relative = y as f64 - view_h as f64 / 2.0;
                let cell = &mut cells[(y + 2) * w + x];
                if relative.abs() <= wall_h / 2.0 {
                    let light = (1.0 / (1.0 + distance * 0.16)) * if side { 0.68 } else { 1.0 };
                    let mortar =
                        texture < 0.035 || ((relative / wall_h + 0.5) * 8.0).fract() < 0.055;
                    cell.glyph = if mortar {
                        ':'
                    } else if distance < 2.0 {
                        '#'
                    } else if distance < 5.0 {
                        '+'
                    } else {
                        '.'
                    };
                    cell.color = Color::Rgb {
                        r: (110.0 * light) as u8,
                        g: (190.0 * light) as u8,
                        b: (210.0 * light) as u8,
                    };
                } else if relative > 0.0 {
                    cell.glyph = if (x + y * 3) % 11 == 0 { '.' } else { ' ' };
                    cell.color = Color::DarkGrey;
                }
            }
        }
        // Project a green beacon into the scene, with wall occlusion.
        let gx = game.maze.goal().0 as f64 + 0.5 - game.x;
        let gy = game.maze.goal().1 as f64 + 0.5 - game.y;
        let forward = gx * game.angle.cos() + gy * game.angle.sin();
        let lateral = -gx * game.angle.sin() + gy * game.angle.cos();
        if forward > 0.1 {
            let center = w as f64 / 2.0 + lateral / forward * w as f64 / (2.0 * (fov / 2.0).tan());
            let beacon_h = (projection / forward * 0.8).min(view_h as f64);
            for (x, &wall_depth) in depth.iter().enumerate() {
                if (x as f64 - center).abs() < (beacon_h * 0.3).max(0.7) && forward < wall_depth {
                    for y in 0..view_h {
                        if (y as f64 - view_h as f64 / 2.0).abs() < beacon_h / 2.0 {
                            cells[(y + 2) * w + x] = Cell {
                                glyph: '*',
                                color: GREEN,
                            };
                        }
                    }
                }
            }
        }
        text(
            &mut cells,
            w,
            2,
            0,
            &format!(
                "{:02}:{:02}    Moves {}",
                elapsed.as_secs() / 60,
                elapsed.as_secs() % 60,
                game.steps
            ),
            Color::Grey,
        );
        let compass_x = w / 2 - 4;
        for (y, row) in [
            "    N    ",
            "         ",
            "W   +   E",
            "         ",
            "    S    ",
        ]
        .into_iter()
        .enumerate()
        {
            text(&mut cells, w, compass_x, y, row, Color::Grey);
        }
        let (dx, dy, pointer) = [
            (2, 0, ">"),
            (2, 1, "\\"),
            (0, 1, "v"),
            (-2, 1, "/"),
            (-2, 0, "<"),
            (-2, -1, "\\"),
            (0, -1, "^"),
            (2, -1, "/"),
        ][((game.angle / std::f64::consts::FRAC_PI_4).round() as usize) % 8];
        text(
            &mut cells,
            w,
            (compass_x as isize + 4 + dx) as usize,
            (2 + dy) as usize,
            pointer,
            GOLD,
        );
        if map && h >= SIZE + 9 && w >= 85 {
            let left = w - SIZE - 3;
            text(&mut cells, w, left, 2, " EXPLORED / @ YOU", GOLD);
            for y in 0..SIZE {
                for x in 0..SIZE {
                    let (glyph, color) = if (x, y) == (game.x as usize, game.y as usize) {
                        ('@', GOLD)
                    } else if !game.seen[y][x] {
                        (' ', Color::DarkGrey)
                    } else if (x, y) == game.maze.goal() {
                        ('X', GREEN)
                    } else if game.maze.is_wall(x as isize, y as isize) {
                        ('#', Color::DarkGrey)
                    } else {
                        ('.', Color::Grey)
                    };
                    cells[(y + 3) * w + left + x] = Cell { glyph, color };
                }
            }
        }
        text(
            &mut cells,
            w,
            2,
            h - 2,
            "Up/Down Walk  Left/Right Turn  M Map  R New  Esc Quit",
            Color::Grey,
        );
        if game.won {
            let message = " EXIT FOUND!  R: explore a new maze  Esc: quit ";
            let left = (w - message.len()) / 2;
            for y in h / 2 - 1..=h / 2 + 1 {
                text(&mut cells, w, left, y, &" ".repeat(message.len()), GREEN);
            }
            text(&mut cells, w, left, h / 2, message, GREEN);
        } else if map && (h < SIZE + 9 || w < 85) {
            text(
                &mut cells,
                w,
                2,
                h - 3,
                "Map needs 86 x 30 terminal. M: toggle",
                GOLD,
            );
        }
    }
    queue!(out, SetBackgroundColor(SKY))?;
    let mut color = None;
    for y in 0..h {
        queue!(out, MoveTo(0, y as u16))?;
        for cell in &cells[y * w..(y + 1) * w] {
            if color != Some(cell.color) {
                queue!(out, SetForegroundColor(cell.color))?;
                color = Some(cell.color);
            }
            queue!(out, Print(cell.glyph))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_small_normal_and_winning_screens() {
        let mut game = Game::new(crate::generator::generate(42));
        for (width, height) in [(0, 0), (1, 1), (40, 10), (59, 20), (100, 32), (160, 48)] {
            let mut output = Vec::new();
            draw(&mut output, &game, width, height, true, Duration::ZERO).unwrap();
        }
        game.won = true;
        let mut output = Vec::new();
        draw(&mut output, &game, 100, 32, true, Duration::ZERO).unwrap();
        assert!(String::from_utf8(output).unwrap().contains("EXIT FOUND!"));
    }
}
