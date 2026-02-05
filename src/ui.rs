use ratatui::{Frame, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, Clear, Paragraph}};

use crate::app::{App, InputMode};


pub fn ui(frame: &mut Frame, app: &App) {

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let modified_indicator = if app.modified { " ●" } else { "" };
    

    let title = Paragraph::new(Line::from(vec![
        Span::styled("Oxide", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(&app.filename, Style::default().fg(Color::White)),
        Span::styled(modified_indicator, Style::default().fg(Color::Yellow)),
    ])).alignment(Alignment::Center).style(Style::default().bg(Color::Rgb(30, 30, 30)));

    let editor_block = Block::default().borders(Borders::ALL).style(Style::default());
  
    let line_number_width = app.content.len().to_string().len().max(2);
    
    let editor_text: Vec<Line> = app.content.iter().enumerate().map(|(i, line)| {
        let is_cursor_line = i == app.cursor_row;
        let line_num_style = if is_cursor_line {
            Style::default().fg(Color::Yellow)

        } else {
            Style::default().fg(Color::DarkGray)

        };
 
        let mut spans = vec![
            Span::styled(format!(" {:>width$} │ ", i + 1, width = line_number_width),line_num_style,),
        ];

        let sel = app.selection.map(|s| app.normalize_selection(s));
        let in_selection = |row: usize, col: usize| -> bool {
            if let Some((sr, sc, er, ec)) = sel {
                if row > sr && row < er { return true; }

                if row == sr && row == er { return col >= sc && col < ec; }

                if row == sr { return col >= sc; }

                if row == er { return col < ec; }
            }
            false
        };


        if is_cursor_line || sel.map_or(false, |(sr, _, er, _)| i >= sr && i <= er) {
            let mut col = 0;
            for ch in line.chars() {
                let is_cursor = is_cursor_line && col == app.cursor_col;

                let is_selected = in_selection(i, col);
                
                let style = if is_cursor {
                    Style::default().bg(Color::White).fg(Color::Black)
                } else if is_selected {
                    Style::default().bg(Color::Rgb(60, 60, 120)).fg(Color::White)
                } else {
                    Style::default()
                };

                spans.push(Span::styled(ch.to_string(), style));
                col += 1;
            }
            if is_cursor_line && app.cursor_col >= line.len() {
                spans.push(Span::styled(" ", Style::default().bg(Color::White).fg(Color::Black)));
            }
        } else {
            spans.push(Span::raw(line.as_str()));

        }

        Line::from(spans)
    }).collect();


    let editor = Paragraph::new(editor_text)
        .block(editor_block)
        .scroll((app.scroll_offset as u16, 0));

    let key_style = Style::default().fg(Color::Rgb(30, 30, 30)).bg(Color::Rgb(100, 100, 100)).add_modifier(Modifier::BOLD);
    
    let label_style = Style::default().fg(Color::Rgb(200, 200, 200));
    
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ^S ", key_style),
        Span::styled("Save ", label_style),
        Span::styled(" ^F ", key_style),
        Span::styled("Find ", label_style),
        Span::styled(" ^G ", key_style),
        Span::styled("GoTo ", label_style),
        Span::styled(" ^Z ", key_style),
        Span::styled("Undo ", label_style),
        Span::styled(" ^Q ", key_style),
        Span::styled("Quit ", label_style),
        Span::styled(
            format!(" Ln {}, Col {} ", app.cursor_row + 1, app.cursor_col + 1),
            Style::default().fg(Color::Rgb(150, 150, 150)),
        ),
    ]))
    .style(Style::default().bg(Color::Rgb(45, 45, 45)));

    frame.render_widget(title, chunks[0]);
    frame.render_widget(editor, chunks[1]);
    frame.render_widget(footer, chunks[2]);



    match app.input_mode {
        InputMode::Search => {

            let area = centered_rect(50, 3, frame.area());
            frame.render_widget(Clear, area);

            let search_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(40, 40, 40)));

            let search_text = Paragraph::new(Line::from(vec![
                Span::styled("Search: ", Style::default().fg(Color::Cyan)),
                Span::styled(&app.input_buffer, Style::default().fg(Color::White)),
                Span::styled("▌", Style::default().fg(Color::White)),
            ])).block(search_block);


            frame.render_widget(search_text, area);
        }
        InputMode::GoToLine => {

            let area = centered_rect(30, 3, frame.area());
            frame.render_widget(Clear, area);

            let goto_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(40, 40, 40)));
            let goto_text = Paragraph::new(Line::from(vec![
                Span::styled("Go to line: ", Style::default().fg(Color::Cyan)),
                Span::styled(&app.input_buffer, Style::default().fg(Color::White)),
                Span::styled("▌", Style::default().fg(Color::White)),
            ])).block(goto_block);

            frame.render_widget(goto_text, area);
        }
        InputMode::ConfirmQuit => {


            let area = centered_rect(45, 3, frame.area());
            frame.render_widget(Clear, area);
            
            let confirm_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(60, 40, 40)));
            let confirm_text = Paragraph::new(Line::from(vec![
                Span::styled("Unsaved changes! Quit? (y/n)", Style::default().fg(Color::Yellow)),
            ])).block(confirm_block).alignment(Alignment::Center);

            frame.render_widget(confirm_text, area);
        }

        InputMode::Normal => {}
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    Rect::new(x, y, width.min(area.width), height.min(area.height))
}