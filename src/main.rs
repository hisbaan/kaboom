mod app;
mod ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::{
    collections::HashSet,
    fs::read_to_string,
    io::{self, Result},
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use app::{ActiveScreen, App, Config, Gamemode, Input, StatefulList};
use ui::{game, title};

/// TODO
/// - title screen
/// - read from config into a struct that is merged with default
/// - settings menu that edits config file
/// - if all letters are used, lives += 1 up to max lives
///
/// - Write script to package instead of having to remember the same process over and over

fn main() -> Result<()> {
    // setup a terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Start game
    let app = init_app();
    let tick_rate = Duration::from_millis(17);
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err)
    }
    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<()> {
    app.title_list.select(0);
    app.pause_list.select(0);
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| match app.active_screen {
            ActiveScreen::Title => title(f, &mut app),
            ActiveScreen::Game => game(f, &mut app),
            _ => {}
        })?;

        match app.active_screen {
            ActiveScreen::Title => {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.title_list.up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.title_list.down();
                        }
                        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                            match app.title_list.state.selected() {
                                Some(i) => {
                                    if app.title_list.items[i] == "Quit" {
                                        return Ok(());
                                    }
                                    match app.title_list.items[i] {
                                        "Start" => {
                                            app.active_screen = ActiveScreen::Game;
                                            start_game(&mut app);
                                        }
                                        "Settings" => app.active_screen = ActiveScreen::Settings,
                                        _ => {}
                                    }
                                }
                                None => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            ActiveScreen::Game => {
                // TODO make button presses override tick rate in terms of drawing screen so the UI doesn't feel laggy
                // basically, use the tick rate to only draw the bar if possible.
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));
                if crossterm::event::poll(timeout)? {
                    if let Event::Key(key) = event::read()? {
                        if app.paused {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    app.paused = false;
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    app.pause_list.up();
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    app.pause_list.down();
                                }
                                KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                                    match app.pause_list.state.selected() {
                                        Some(i) => {
                                            if app.pause_list.items[i] == "Quit" {
                                                return Ok(());
                                            }
                                            match app.pause_list.items[i] {
                                                // TODO make these into a tuple and include the function as a closure
                                                "Resume" => app.paused = false,
                                                "Main Menu" => {
                                                    app.active_screen = ActiveScreen::Title
                                                }
                                                // TODO kill game
                                                "Restart" => {
                                                    // kill game then
                                                    start_game(&mut app);
                                                    app.paused = false;
                                                }
                                                _ => {}
                                            }
                                        }
                                        None => {}
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            match key.code {
                                KeyCode::Enter => {
                                    if check_word(&mut app) {
                                        // TODO if correct, should the time stack?
                                        next_turn(&mut app);
                                        app.input.string = "".to_string();
                                    }
                                }
                                KeyCode::Char(c) => {
                                    app.input.string.push(c);
                                }
                                KeyCode::Backspace => {
                                    app.input.string.pop();
                                }
                                KeyCode::Esc => {
                                    app.paused = true;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                if matches!(app.active_screen, ActiveScreen::Game)
                    && !app.paused
                    && !matches!(app.config.gamemode, Gamemode::Practice)
                    && last_tick.elapsed() >= tick_rate
                {
                    if app.time_left >= 1 {
                        app.time_left -= 1;
                    } else {
                        // time up: end word, subtract life
                        if matches!(app.config.gamemode, Gamemode::LimitedLives) {
                            app.lives -= 1;
                            if app.lives == 0 {
                                app.active_screen = ActiveScreen::Title; // TODO ActiveScreen::GameOver;
                            }
                        }
                        next_turn(&mut app);
                    }
                    last_tick = Instant::now();
                }
            }
            _ => {}
        }
    }
}

fn init_app() -> App<'static> {
    // Get word list
    let word_list = read_to_string("src/dict").expect("Error reading file");
    let dictionary: Vec<String> = word_list
        .split('\n')
        .map(|s| -> String { s.strip_suffix("\r").unwrap_or(s).to_string() })
        .collect();
    // let mut dictionary_hash_set = HashSet::new();
    // dictionary_hash_set.insert("AALS".to_string());
    let dictionary_hash_set = HashSet::from_iter(dictionary.clone());

    let config = Config::default();
    let input = Input::default();

    App {
        active_screen: ActiveScreen::Title,
        // TODO replace these list strings with an enum of sort
        title_list: StatefulList::with_items(vec!["Start", "Settings", "Quit"]),
        input,
        prompt: "".to_string(),
        time_left: config.time_per_turn * 64,
        lives: config.starting_lives + 1,
        paused: false,
        pause_list: StatefulList::with_items(vec!["Resume", "Main Menu", "Restart", "Quit"]),
        dictionary,
        dictionary_hash_set,
        config,
    }
}

fn start_game(app: &mut App) {
    // set lives, other options
    app.lives = app.config.starting_lives + 1;
    next_turn(app);
}

fn next_turn(app: &mut App) {
    app.prompt = generate_prompt(app, app.config.min_wpp);
    app.input.string = "".to_string();
    app.time_left = app.config.time_per_turn * 64;
}

fn generate_prompt(app: &mut App, wpp: usize) -> String {
    let mut rng = rand::thread_rng();
    let coin = rng.gen::<f64>();
    let upper_bound = if coin > 0.8 { 3 } else { 2 };
    loop {
        let mut prompt = "".to_string();
        for _ in 0..upper_bound {
            prompt.push(rng.gen_range(b'A'..b'Z') as char)
        }
        let mut counter = 0;
        for word in &app.dictionary {
            if word.contains(&prompt) {
                counter += 1;
            }
            if counter >= wpp {
                return prompt;
            }
        }
    }
}

fn check_word(app: &mut App) -> bool {
    let word = &app.input.string.to_uppercase();
    word.contains(&app.prompt) && app.dictionary_hash_set.contains(word)
}
