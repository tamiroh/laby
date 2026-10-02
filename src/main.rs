mod game;
mod render;

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use game::Game;
use std::{
    io::{self, IsTerminal, Write},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct Terminal;
impl Terminal {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let guard = Self;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(guard)
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        restore();
    }
}
fn restore() {
    let _ = execute!(
        io::stdout(),
        crossterm::style::ResetColor,
        Show,
        LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
}
fn seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "LABY — terminal maze explorer\n\nUsage: laby [--seed NUMBER]\n\nUp/Down: walk   Left/Right: turn\nM: map   R: new maze   Esc/Ctrl-C: quit\n\nFind the green exit. Recommended terminal: 100 x 32 or larger."
        );
        return Ok(());
    }
    let initial_seed = match args.as_slice() {
        [] => seed(),
        [flag, value] if flag == "--seed" => value.parse().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "seed must be an unsigned integer",
            )
        })?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Usage: laby [--seed NUMBER]",
            ));
        }
    };
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::other(
            "Run laby in an interactive terminal (cargo run --release).",
        ));
    }
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        hook(info);
    }));
    let _terminal = Terminal::enter()?;
    let mut game = Game::new(initial_seed);
    let mut started = Instant::now();
    let mut finished = None;
    let mut map = true;
    let mut stdout = io::BufWriter::new(io::stdout());
    loop {
        let (width, height) = terminal::size()?;
        render::draw(
            &mut stdout,
            &game,
            width,
            height,
            map,
            finished.unwrap_or_else(|| started.elapsed()),
        )?;
        stdout.flush()?;
        if !event::poll(Duration::from_millis(50))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            if key.code == KeyCode::Esc
                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
            {
                break;
            }
            match key.code {
                KeyCode::Up => game.advance(1.0, 0.0),
                KeyCode::Down => game.advance(-1.0, 0.0),
                KeyCode::Left => game.turn(-0.13),
                KeyCode::Right => game.turn(0.13),
                KeyCode::Char('m' | 'M') => map = !map,
                KeyCode::Char('r' | 'R') => {
                    game = Game::new(seed());
                    started = Instant::now();
                    finished = None;
                }
                _ => {}
            }
            if game.won && finished.is_none() {
                finished = Some(started.elapsed());
            }
        }
    }
    Ok(())
}
