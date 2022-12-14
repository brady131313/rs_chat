use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{actions::Actions, state::Pane, App};

pub fn draw<B: Backend>(rect: &mut Frame<B>, app: &mut App, username: &str) {
    let size = rect.size();
    if size.width < 87 || size.height < 22 {
        panic!("Screen too small");
    }

    let block = Block::default()
        .title(Spans::from(vec![
            Span::from("IRC as "),
            current_user_span(username),
        ]))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    rect.render_widget(block, size);

    let app_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(15), Constraint::Length(4)])
        .split(size);

    let action_menu = actions_menu(app.current_actions());
    rect.render_widget(action_menu, app_chunks[1]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(app_chunks[0]);

    let active_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[0]);

    // Active Rooms
    let active_rooms: Vec<ListItem> = app
        .state
        .active_rooms
        .items
        .iter()
        .map(|i| ListItem::new(Span::from(i.as_ref())))
        .collect();

    let active_rooms = List::new(active_rooms)
        .block(panel(Pane::Rooms, app.state.current_pane()))
        .highlight_style(Style::default().fg(Color::LightBlue))
        .highlight_symbol("> ");
    rect.render_stateful_widget(
        active_rooms,
        active_chunk[0],
        &mut app.state.active_rooms.state,
    );

    // Active Chats
    let active_chats: Vec<ListItem> = app
        .state
        .active_chats
        .items
        .iter()
        .map(|i| ListItem::new(Span::from(i.as_ref())))
        .collect();

    let active_chats = List::new(active_chats)
        .block(panel(Pane::Chats, app.state.current_pane()))
        .highlight_style(Style::default().fg(Color::LightBlue))
        .highlight_symbol("> ");
    rect.render_stateful_widget(
        active_chats,
        active_chunk[1],
        &mut app.state.active_chats.state,
    );

    // Messages
    let message_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(85), Constraint::Percentage(15)])
        .split(chunks[1]);

    let messages_block = panel(Pane::Messages, app.state.current_pane());
    let new_message_block = panel(Pane::NewMessage, app.state.current_pane());

    if let Some(messages) = app.state.current_messages_mut() {
        let message_items: Vec<_> = messages
            .items
            .iter()
            .map(|m| message_list_item(m, username))
            .collect();

        let message_list = List::new(message_items)
            .block(messages_block)
            .highlight_symbol("> ");

        rect.render_stateful_widget(message_list, message_chunks[0], &mut messages.state);

        let message_input = Paragraph::new(app.state.new_message.as_str())
            .block(new_message_block)
            .wrap(Wrap { trim: false });
        rect.render_widget(message_input, message_chunks[1]);
    } else {
        rect.render_widget(messages_block, message_chunks[0]);
        rect.render_widget(new_message_block, message_chunks[1]);
    }

    // Room Users
    let users = panel(Pane::Users, app.state.current_pane());
    if let Some(room_users) = app.state.current_room_users_mut() {
        let list_items: Vec<_> = room_users
            .items
            .iter()
            .map(|i| user_list_item(i, username))
            .collect();

        let list = List::new(list_items)
            .block(users)
            .highlight_style(Style::default().fg(Color::LightBlue))
            .highlight_symbol("> ");

        rect.render_stateful_widget(list, chunks[2], &mut room_users.state);
    } else {
        rect.render_widget(users, chunks[2]);
    }

    match app.state.current_pane() {
        Pane::NewRoom => {
            let block = panel(Pane::NewRoom, app.state.current_pane());
            let area = centered_rect(60, 12, size);
            let input = Paragraph::new(app.state.new_room.as_str()).block(block);
            rect.render_widget(Clear, area);
            rect.render_widget(input, area);
        }
        Pane::AllUsers => {
            let area = centered_rect(45, 30, size);
            rect.render_widget(Clear, area);

            let all_users: Vec<ListItem> = app
                .state
                .all_users
                .items
                .iter()
                .map(|i| user_list_item(i, username))
                .collect();

            let all_users = List::new(all_users)
                .block(panel(Pane::AllUsers, app.state.current_pane()))
                .highlight_style(Style::default().fg(Color::LightBlue))
                .highlight_symbol("> ");

            rect.render_stateful_widget(all_users, area, &mut app.state.all_users.state);
        }
        Pane::AllRooms => {
            let area = centered_rect(45, 30, size);
            rect.render_widget(Clear, area);

            let all_rooms: Vec<ListItem> = app
                .state
                .all_rooms
                .items
                .iter()
                .map(|i| ListItem::new(Span::from(i.as_ref())))
                .collect();

            let all_rooms = List::new(all_rooms)
                .block(panel(Pane::AllRooms, app.state.current_pane()))
                .highlight_style(Style::default().fg(Color::LightBlue))
                .highlight_symbol("> ");

            rect.render_stateful_widget(all_rooms, area, &mut app.state.all_rooms.state);
        }
        _ => {}
    }
}

fn actions_menu(actions: &Actions) -> Paragraph {
    let mut spans: Vec<Span> = vec![];

    let mut iter = actions.actions().iter();
    if let Some(action) = iter.next() {
        spans.push(Span::from(action.display_with_keys()));

        for action in iter {
            spans.push(Span::from("   "));
            spans.push(Span::from(action.display_with_keys()));
        }
    }
    Paragraph::new(Spans::from(spans))
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true })
}

fn message_list_item<'a>(current: &'a (String, String), username: &'a str) -> ListItem<'a> {
    let sender_span = if current.0 == username {
        current_user_span(username)
    } else {
        Span::from(current.0.as_str())
    };

    ListItem::new(Spans::from(vec![
        sender_span,
        Span::from(format!(": {}", current.1)),
    ]))
}

fn user_list_item<'a>(current: &'a str, username: &'a str) -> ListItem<'a> {
    if current == username {
        ListItem::new(current_user_span(username))
    } else {
        ListItem::new(Span::from(current))
    }
}

fn current_user_span(username: &str) -> Span {
    Span::styled(username, Style::default().add_modifier(Modifier::ITALIC))
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
