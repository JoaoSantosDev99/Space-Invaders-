use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use invaders::{
    frame::{self, new_frame, Drawable},
    player::Player,
    render,
};
use rusty_audio::Audio;
use std::{
    error::Error,
    io::{self},
    sync::mpsc,
    time::Duration,
};
use std::{
    thread::{self, spawn},
    time::Instant,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    audio.add("explode", "./src/assets/sounds/explode.wav");
    audio.add("lose", "./src/assets/sounds/lose.wav");
    audio.add("move", "./src/assets/sounds/move.wav");
    audio.add("pew", "./src/assets/sounds/pew.wav");
    audio.add("startup", "./src/assets/sounds/startup.wav");
    audio.add("win", "./src/assets/sounds/win.wav");
    audio.play("startup");

    // terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?; // pops a new terminal window;
    stdout.execute(Hide)?; // hides the coursor

    // Render loop in a separeted thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };

            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Game loop
    let mut player = Player::new();
    let mut instant = Instant::now();

    'gameloop: loop {
        // Per-frame init
        let mut curr_frame = new_frame();
        let delta = instant.elapsed();
        instant = Instant::now();
        // Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Up => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {
                        audio.play("pew");
                    }
                }
            }
        }

        // Updates
        player.update(delta);

        // Draw and render
        player.draw(&mut curr_frame);
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
    }

    // Clean up
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
