use colored::Colorize;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, Gamemode};

pub fn title<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2) // TODO make this a function of terminal size? Percentage?
        .constraints(
            [
                Constraint::Length(5),
                Constraint::Length(2),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let text = vec![
        Spans::from(" _  __     _                          "),
        Spans::from("| |/ /__ _| |__   ___   ___  _ __ __  "),
        Spans::from("| ' // _` | '_ \\ / _ \\ / _ \\| '_ ` _ \\"),
        Spans::from("| . \\ (_| | |_) | (_) | (_) | | | | | |"),
        Spans::from("|_|\\_\\__,_|_.__/ \\___/ \\___/|_| |_| |_|"),
    ];
    let title = Paragraph::new(text).alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = app
        .title_list
        .items
        .iter()
        .map(|i| ListItem::new(vec![Spans::from(i.to_owned())]).style(Style::default()))
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(&app.config.list_hightlight_symbol);

    f.render_stateful_widget(items, chunks[2], &mut app.title_list.state);
}

pub fn game<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    // prompt
    let prompt = Paragraph::new(app.prompt.to_owned())
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(prompt, chunks[0]);

    // time remaining bar
    let max_progress = (f.size().width - 2/* margin */) * 8;
    let curr_progress = max_progress as f32 * (app.time_left as f32 / 320.0);
    let progress = Paragraph::new(Text::from(Span::styled(
        "█".repeat(curr_progress as usize / 8)
            + match curr_progress as usize % 8 {
                7 => "▉",
                6 => "▊",
                5 => "▋",
                4 => "▌",
                3 => "▍",
                2 => "▎",
                1 => "▏",
                _ => "",
            },
        Style::default(),
    )));
    f.render_widget(progress, chunks[1]);

    // input
    // TODO color substring equaling prompt green
    let input_field = Paragraph::new(app.input.string.as_ref())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(input_field, chunks[2]);
    f.set_cursor(
        chunks[2].x + app.input.string.len() as u16 + 1,
        chunks[2].y + 1,
    );

    let hearts_text = match app.config.gamemode {
        Gamemode::Practice => "practice".to_string(),
        Gamemode::LimitedLives => "".repeat(app.lives),
        Gamemode::InfiniteLives => "∞".to_string(),
    };
    let hearts = Paragraph::new(hearts_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red));
    f.render_widget(hearts, chunks[3]);

    if app.paused {
        // NOTE this only seems to work well in full screen
        let items: Vec<ListItem> = app
            .pause_list
            .items
            .iter()
            .map(|i| ListItem::new(vec![Spans::from(i.to_owned())]).style(Style::default()))
            .collect();

        let items = List::new(items)
            .block(
                Block::default()
                    .title("Paused")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(&app.config.list_hightlight_symbol);

        let area = centered_rect(40, 60, f.size());
        f.render_widget(Clear, area);
        f.render_stateful_widget(items, area, &mut app.pause_list.state);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
