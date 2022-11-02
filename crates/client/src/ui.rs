use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, Pane};

pub fn draw<B: Backend>(rect: &mut Frame<B>, app: &mut App, username: &str) {
    let size = rect.size();

    let block = Block::default()
        .title(Spans::from(vec![
            Span::from("IRC as "),
            Span::styled(username, Style::default().add_modifier(Modifier::ITALIC)),
        ]))
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
    let users = panel(Pane::Users, app.current_pane());
    if let Some(room_users) = app.current_room_users_mut() {
        let list_items: Vec<_> = room_users
            .items
            .iter()
            .map(|i| {
                if i == username {
                    ListItem::new(Span::styled(
                        i,
                        Style::default().add_modifier(Modifier::ITALIC),
                    ))
                } else {
                    ListItem::new(Span::from(i.as_ref()))
                }
            })
            .collect();

        let list = List::new(list_items)
            .block(users)
            .highlight_style(Style::default().fg(Color::LightBlue))
            .highlight_symbol("> ");

        rect.render_stateful_widget(list, chunks[2], &mut room_users.state);
    } else {
        rect.render_widget(users, chunks[2]);
    }

    match app.current_pane() {
        Pane::NewRoom => {
            let block = panel(Pane::NewRoom, app.current_pane());
            let area = centered_rect(60, 5, size);
            let input = Paragraph::new(app.new_room());
            rect.render_widget(Clear, area);
            rect.render_widget(input, area);
        }
        Pane::AllUsers => {
            let area = centered_rect(60, 30, size);
            rect.render_widget(Clear, area);

            let all_users: Vec<ListItem> = app
                .all_users
                .items
                .iter()
                .map(|i| {
                    if i == username {
                        ListItem::new(Span::styled(
                            i,
                            Style::default().add_modifier(Modifier::ITALIC),
                        ))
                    } else {
                        ListItem::new(Span::from(i.as_ref()))
                    }
                })
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
