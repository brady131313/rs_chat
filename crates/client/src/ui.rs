use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem},
    Frame,
};

use crate::app::{App, Pane};

pub fn draw<B: Backend>(rect: &mut Frame<B>, app: &mut App) {
    let size = rect.size();

    let block = Block::default()
        .title("IRC")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    rect.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(size);

    // Active Rooms
    let active_rooms: Vec<ListItem> = app
        .active_rooms
        .items
        .iter()
        .map(|i| ListItem::new(Span::from(i.as_ref())))
        .collect();

    let active_rooms = List::new(active_rooms)
        .block(panel(Pane::Rooms, app.current_pane()))
        .highlight_style(Style::default().fg(Color::LightBlue))
        .highlight_symbol("> ");
    rect.render_stateful_widget(active_rooms, chunks[0], &mut app.active_rooms.state);

    let messages_block = panel(Pane::Messages, app.current_pane());
    rect.render_widget(messages_block, chunks[1]);

    // Room Users
    let room_users: Vec<ListItem> = app
        .room_users
        .items
        .iter()
        .map(|i| ListItem::new(Span::from(i.as_ref())))
        .collect();

    let room_users = List::new(room_users)
        .block(panel(Pane::Users, app.current_pane()))
        .highlight_style(Style::default().fg(Color::LightBlue))
        .highlight_symbol("> ");
    rect.render_stateful_widget(room_users, chunks[2], &mut app.room_users.state);

    match app.current_pane() {
        Pane::NewRoom => {
            let block = panel(Pane::NewRoom, app.current_pane());
            let area = centered_rect(60, 5, size);
            rect.render_widget(Clear, area);
            rect.render_widget(block, area);
        }
        Pane::AllUsers => {
            let area = centered_rect(60, 30, size);
            rect.render_widget(Clear, area);

            let all_users: Vec<ListItem> = app
                .all_users
                .items
                .iter()
                .map(|i| ListItem::new(Span::from(i.as_ref())))
                .collect();

            let all_users = List::new(all_users)
                .block(panel(Pane::AllUsers, app.current_pane()))
                .highlight_style(Style::default().fg(Color::LightBlue))
                .highlight_symbol("> ");

            rect.render_stateful_widget(all_users, area, &mut app.all_users.state);
        }
        Pane::AllRooms => {
            let area = centered_rect(60, 30, size);
            rect.render_widget(Clear, area);

            let all_rooms: Vec<ListItem> = app
                .all_rooms
                .items
                .iter()
                .map(|i| ListItem::new(Span::from(i.as_ref())))
                .collect();

            let all_rooms = List::new(all_rooms)
                .block(panel(Pane::AllRooms, app.current_pane()))
                .highlight_style(Style::default().fg(Color::LightBlue))
                .highlight_symbol("> ");

            rect.render_stateful_widget(all_rooms, area, &mut app.all_rooms.state);
        }
        _ => {}
    }
}

fn panel(pane: Pane, active: Pane) -> Block<'static> {
    Block::default()
        .title(pane.title())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if pane == active {
            Color::LightBlue
        } else {
            Color::White
        }))
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
