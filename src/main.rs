use std::fs;
use std::io;


mod app;
mod ui;

use std::error::Error;
use clap::Parser;

use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}};
use ratatui::{Terminal, prelude::{Backend, CrosstermBackend}};
use crate::ui::ui;

use crate::app::{App, InputMode, Selection};

#[derive(Parser, Debug)]
struct Args {
    filename: String
}



fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut app = App::new();
    app.filename = args.filename.clone();
    
    match fs::read_to_string(&args.filename) {
        Ok(contents) => {

            app.content = contents.lines().map(String::from).collect();

            if app.content.is_empty() {
                app.content.push(String::new());
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            app.content.push(String::new());
            app.modified = true;
        }

        Err(e) => return Err(format!("Cannot open '{}': {}", args.filename, e).into()),
    }

    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal, &mut app)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

    terminal.show_cursor()?;

    Ok(())

}


fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>>
where
    B::Error: 'static,
{
    terminal.clear()?;
    
    loop {

        let viewport_height = terminal.size()?.height.saturating_sub(4) as usize;
        app.adjust_scroll(viewport_height);
        
        let _ = terminal.draw(|f| ui(f, app));

        match event::read()? {


            Event::Key(key) => {
                match app.input_mode {

                    InputMode::Normal => {
                        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                        
                        match key.code {

                            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                if app.modified {
                                    app.input_mode = InputMode::ConfirmQuit;
                                } else {
                                    break;
                                }
                            }

                            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                let mut content = app.content.join("\n");
                                content.push('\n');
                                if fs::write(&app.filename, &content).is_ok() {
                                    app.modified = false;
                                }
                            }


                            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.undo();
                            }

                            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.redo();
                            }

                            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.input_mode = InputMode::Search;
                                app.input_buffer.clear();
                            }


                            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.input_mode = InputMode::GoToLine;
                                app.input_buffer.clear();
                            }

                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                if app.selection.is_some() {
                                    app.clipboard = app.get_selected_text();
                                }
                            }
                            KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                if app.selection.is_some() {
                                    app.clipboard = app.get_selected_text();
                                    app.delete_selection();
                                }
                            }
                        
                
                            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                if !app.clipboard.is_empty() {
                                    app.save_state();
                                    if app.selection.is_some() {
                                        app.delete_selection();
                                    }

                                    for (i, chunk) in app.clipboard.clone().split('\n').enumerate() {
                                        if i > 0 {
                                            let line = app.content.get_mut(app.cursor_row).unwrap();
                                            let new_line = line.split_off(app.cursor_col);

                                            app.cursor_row += 1;
                                            app.cursor_col = 0;

                                            app.content.insert(app.cursor_row, new_line);
                                        }
                                        if let Some(line) = app.content.get_mut(app.cursor_row) {
                                            line.insert_str(app.cursor_col, chunk);
                                            app.cursor_col += chunk.len();


                                        }
                                    }
                                    app.modified = true;

                                }
                            }

                            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.selection = Some(Selection {
                                    start_row: 0,
                                    start_col: 0,
                                    end_row: app.content.len().saturating_sub(1),
                                    end_col: app.content.last().map_or(0, |l| l.len()),
                                });
                            }

                            KeyCode::Up => {
                                if shift && app.selection.is_none() {
                                    app.start_selection();
                                } else if !shift {
                                    app.clear_selection();
                                }
                                if app.cursor_row > 0 {
                                    app.cursor_row -= 1;
                                    let line_len = app.content.get(app.cursor_row).map_or(0, |l| l.len());
                                    app.cursor_col = app.cursor_col.min(line_len);
                                }
                                if shift { app.update_selection(); }
                            }
                            KeyCode::Down => {
                                if shift && app.selection.is_none() {
                                    app.start_selection();
                                } else if !shift {
                                    app.clear_selection();
                                }
                                if app.cursor_row < app.content.len().saturating_sub(1) {
                                    app.cursor_row += 1;
                                    let line_len = app.content.get(app.cursor_row).map_or(0, |l| l.len());
                                    app.cursor_col = app.cursor_col.min(line_len);
                                }
                                if shift { app.update_selection(); }
                            }

                            KeyCode::Left => {
                                if shift && app.selection.is_none() { app.start_selection(); } else if !shift { app.clear_selection(); }
                                if app.cursor_col > 0 {
                                    app.cursor_col -= 1;

                                } else if app.cursor_row > 0 {
                                    app.cursor_row -= 1;
                                    app.cursor_col = app.content.get(app.cursor_row).map_or(0, |l| l.len());
                                }

                                if shift { app.update_selection(); }
                            }
                            KeyCode::Right => {

                                if shift && app.selection.is_none() {
                                    app.start_selection();
                                } else if !shift {
                                    app.clear_selection();
                                }

                                let line_len = app.content.get(app.cursor_row).map_or(0, |l| l.len());
                                if app.cursor_col < line_len {
                                    app.cursor_col += 1;
                                } else if app.cursor_row < app.content.len().saturating_sub(1) {
                                    app.cursor_row += 1;
                                    app.cursor_col = 0;
                                }

                                if shift { app.update_selection(); }
                            }

                            KeyCode::Home => {
                                if shift && app.selection.is_none() { app.start_selection(); }
                                else if !shift { app.clear_selection(); }
                                app.cursor_col = 0;

                                if shift { app.update_selection(); }
                            }
                            KeyCode::End => {
                                if shift && app.selection.is_none() { app.start_selection(); }
                                else if !shift { app.clear_selection(); }

                                app.cursor_col = app.content.get(app.cursor_row).map_or(0, |l| l.len());
                                if shift { app.update_selection(); }
                            }
                            KeyCode::PageUp => {
                                app.clear_selection();
                                app.cursor_row = app.cursor_row.saturating_sub(viewport_height);
                                let line_len = app.content.get(app.cursor_row).map_or(0, |l| l.len());
                                app.cursor_col = app.cursor_col.min(line_len);
                            }


                            KeyCode::PageDown => {
                                app.clear_selection();
                                app.cursor_row = (app.cursor_row + viewport_height).min(app.content.len().saturating_sub(1));
                                let line_len = app.content.get(app.cursor_row).map_or(0, |l| l.len());
                                app.cursor_col = app.cursor_col.min(line_len);
                            }
                            KeyCode::Tab => {

                                app.save_state();
                                if app.selection.is_some() {
                                    app.delete_selection();

                                }
                                if let Some(line) = app.content.get_mut(app.cursor_row) {
                                    line.insert_str(app.cursor_col, "    ");
                                    app.cursor_col += 4;
                                    app.modified = true;

                                }

                            }
                            KeyCode::Delete => {
                                if app.selection.is_some() { app.delete_selection();
                                } else {
                                    let line_len = app.content.get(app.cursor_row).map_or(0, |l| l.len());
                                    if app.cursor_col < line_len {
                                        

                                        app.save_state();
                                        if let Some(line) = app.content.get_mut(app.cursor_row) {
                                            line.remove(app.cursor_col);
                                        
                                        }
                                        app.modified = true;
                                    } else if app.cursor_row < app.content.len().saturating_sub(1) {
                                        app.save_state();
                                        let next_line = app.content.remove(app.cursor_row + 1);
                                        app.content[app.cursor_row].push_str(&next_line);
                                        app.modified = true;
                                    }

                                }

                            }
                            KeyCode::Char(c) => {
                                app.save_state();
                                if app.selection.is_some() {
                                    app.delete_selection();
                                }
                                if let Some(line) = app.content.get_mut(app.cursor_row) {
                                    line.insert(app.cursor_col, c);
                                    app.cursor_col += 1;
                                    app.modified = true;
                                }
                            }

                            KeyCode::Backspace => {
                                if app.selection.is_some() {
                                    app.delete_selection();

                                } else {
                                    app.save_state();
                                    if app.cursor_col > 0 {
                                        if let Some(line) = app.content.get_mut(app.cursor_row) {
                                            line.remove(app.cursor_col - 1);
                                            app.cursor_col -= 1;


                                        }
                                    } else if app.cursor_row > 0 {
                                        let current_line = app.content.remove(app.cursor_row);
                                        app.cursor_row -= 1;

                                        app.cursor_col = app.content[app.cursor_row].len();
                                        app.content[app.cursor_row].push_str(&current_line);
                                    }
                                    app.modified = true;
                                }
                            }

                            KeyCode::Enter => {
                                app.save_state();
                                if app.selection.is_some() {
                                    app.delete_selection();
                                }
                                if let Some(line) = app.content.get_mut(app.cursor_row) {
                                    let new_line = line.split_off(app.cursor_col);

                                    app.cursor_row += 1;
                                    app.cursor_col = 0;

                                    app.content.insert(app.cursor_row, new_line);

                                    app.modified = true;


                                }
                            }
                            KeyCode::Esc => {
                                app.clear_selection();
                            }


                            _ => {}

                        }
                    }
                    InputMode::Search => {
                        match key.code {

                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.search_matches.clear();
                            }

                            KeyCode::Enter => {

                                app.search_query = app.input_buffer.clone();
                                app.search();

                                if !app.search_matches.is_empty() {
                                    let (row, col) = app.search_matches[0];
                                    app.cursor_row = row;
                                    app.cursor_col = col;

                                }
                                app.input_mode = InputMode::Normal;
                            }

                            KeyCode::Char(c) => {

                                app.input_buffer.push(c);
                            }
                            KeyCode::Backspace => {
                                app.input_buffer.pop();
                            }
                            KeyCode::Down | KeyCode::Tab => {
                                app.search_query = app.input_buffer.clone();
                                app.search();
                                app.next_match();
                            
                            }
                            
                            
                            KeyCode::Up => {
                                app.search_query = app.input_buffer.clone();
                                app.search();
                                app.prev_match();
                            }
                            
                            _ => {}


                        }
                    }
                    InputMode::GoToLine => {

                        match key.code {
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                            }

                            KeyCode::Enter => {
                                if let Ok(line_num) = app.input_buffer.parse::<usize>() {
                                    let target = line_num.saturating_sub(1).min(app.content.len().saturating_sub(1));
                                    
                                    app.cursor_row = target;
                                    app.cursor_col = 0;
                                    
                                }
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) if c.is_ascii_digit() => {
                                app.input_buffer.push(c);
                            }

                            KeyCode::Backspace => {
                                app.input_buffer.pop();
                            }
                            _ => {}


                        }
                    }
                    InputMode::ConfirmQuit => {
                        match key.code {

                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                break;
                            }

                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {}

                        }
                    }
                }
            }
            Event::Mouse(MouseEvent { kind: MouseEventKind::Down(_), column, row, .. }) => {
                if app.input_mode == InputMode::Normal {
                    let editor_start_row = 1u16;
                    let editor_start_col = (app.content.len().to_string().len() + 5) as u16;
                    
                    if row > editor_start_row && column >= editor_start_col {
                        let clicked_row = (row - editor_start_row - 1) as usize + app.scroll_offset;
                        let clicked_col = (column - editor_start_col) as usize;
                        
                        if clicked_row < app.content.len() {
                            app.cursor_row = clicked_row;
                            app.cursor_col = clicked_col.min(app.content[clicked_row].len());
                            app.clear_selection();

                        }
                    }
                    
                }
            }

            _ => {}
        }
    }



    Ok(())

}