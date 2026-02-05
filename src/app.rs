#[derive(Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    GoToLine,
    ConfirmQuit,
}

#[derive(Clone)]
pub struct EditorState {
    pub content: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Selection {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}


pub struct App {
    pub exit: bool,
    pub filename: String,
    pub content: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub modified: bool,
    pub scroll_offset: usize,
    pub undo_stack: Vec<EditorState>,
    pub redo_stack: Vec<EditorState>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub search_query: String,
    pub search_matches: Vec<(usize, usize)>,
    pub search_index: usize,
    pub selection: Option<Selection>,
    pub clipboard: String,
}

impl App {
    pub fn new() -> App {
        App {
        
            exit: false,
            filename: String::new(),
            content: Vec::new(),
            cursor_row: 0,
            cursor_col: 0,
            modified: false,
            scroll_offset: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            search_query: String::new(),
            search_matches: Vec::new(),
            search_index: 0,
            selection: None,
            clipboard: String::new(),
        }
    }

    pub fn save_state(&mut self) {
        self.undo_stack.push(EditorState {
            content: self.content.clone(),
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,    
        });

        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {

        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(EditorState {
                content: self.content.clone(),  
                cursor_row: self.cursor_row,
                cursor_col: self.cursor_col,    
            });

            self.content = state.content;   
            self.cursor_row = state.cursor_row;
            self.cursor_col = state.cursor_col;
            self.modified = true;
        }
    }

    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(EditorState {
                content: self.content.clone(),
                cursor_row: self.cursor_row,
                cursor_col: self.cursor_col,
            });
            self.content = state.content;
            self.cursor_row = state.cursor_row;
            self.cursor_col = state.cursor_col;
            self.modified = true;
        }
    }


    pub fn adjust_scroll(&mut self, viewport_height: usize) {
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        } else if self.cursor_row >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor_row - viewport_height + 1;
        }
    }

    pub fn search(&mut self) {
        self.search_matches.clear();
        if self.search_query.is_empty() {
            return;
        }
        for (row, line) in self.content.iter().enumerate() {
            let mut start = 0;
            while let Some(col) = line[start..].find(&self.search_query) {
                self.search_matches.push((row, start + col));
                start += col + 1;
            }
        }
        self.search_index = 0;
    }

    pub fn next_match(&mut self) {
        if !self.search_matches.is_empty() {

            self.search_index = (self.search_index + 1) % self.search_matches.len();
            let (row, col) = self.search_matches[self.search_index];
            
            self.cursor_row = row;
            self.cursor_col = col;

        }
    }


    pub fn prev_match(&mut self) {
        if !self.search_matches.is_empty() {
            self.search_index = if self.search_index == 0 {

                self.search_matches.len() - 1   
            } else {

                self.search_index - 1
            };
            let (row, col) = self.search_matches[self.search_index];
            self.cursor_row = row;  
            self.cursor_col = col;  
        }
    }

    pub fn start_selection(&mut self) {

        self.selection = Some(Selection {

            start_row: self.cursor_row, 

            start_col: self.cursor_col,

            end_row: self.cursor_row,
            
            end_col: self.cursor_col,

        });
    }

    pub fn update_selection(&mut self) {

        if let Some(ref mut sel) = self.selection {
            sel.end_row = self.cursor_row;
            sel.end_col = self.cursor_col;

        }
        
    }

    pub fn clear_selection(&mut self) { self.selection = None; }

    pub fn get_selected_text(&self) -> String {

        if let Some(sel) = self.selection {
            let (start_row, start_col, end_row, end_col) = self.normalize_selection(sel);
            
            if start_row == end_row {
                self.content[start_row][start_col..end_col].to_string()
            } else {
                let mut result = self.content[start_row][start_col..].to_string();
                result.push('\n');
                for row in (start_row + 1)..end_row {
                    result.push_str(&self.content[row]);
                    result.push('\n');
                }
                result.push_str(&self.content[end_row][..end_col]);
                result
            }

        } else {
            return String::new()
        }
    }

    pub fn normalize_selection(&self, sel: Selection) -> (usize, usize, usize, usize) {
        if sel.start_row < sel.end_row || (sel.start_row == sel.end_row && sel.start_col <= sel.end_col) {
            (sel.start_row, sel.start_col, sel.end_row, sel.end_col)
        } else {
            (sel.end_row, sel.end_col, sel.start_row, sel.start_col)
        }
    }

    pub fn delete_selection(&mut self) {
        if let Some(sel) = self.selection {
            self.save_state();
            let (start_row, start_col, end_row, end_col) = self.normalize_selection(sel);
            
            if start_row == end_row {
                self.content[start_row].replace_range(start_col..end_col, "");
            } else {
                let end_part = self.content[end_row][end_col..].to_string();
                self.content[start_row].truncate(start_col);
                self.content[start_row].push_str(&end_part);
                
                for _ in (start_row + 1)..=end_row {
                    self.content.remove(start_row + 1);
                }
            }

            self.cursor_row = start_row;
            self.cursor_col = start_col;
            self.selection = None;
            self.modified = true;

        }
    }
}