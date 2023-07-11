use std::collections::HashSet;

use tui::widgets::ListState;

pub struct App<'a> {
    pub active_screen: ActiveScreen,
    pub title_list: StatefulList<&'a str>,
    pub input: Input,
    pub prompt: String,
    pub time_left: usize,
    pub paused: bool,
    pub pause_list: StatefulList<&'a str>,
    pub dictionary: Vec<String>,
    pub dictionary_hash_set: HashSet<String>
}

pub enum ActiveScreen {
    Title,
    Settings,
    Game,
    GameOver,
}

pub struct Input {
    pub string: String,
    pub messages: Vec<String>,
}

impl Default for Input {
    fn default() -> Input {
        Input {
            string: String::new(),
            messages: Vec::new(),
        }
    }
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select(&mut self, index: usize) {
        self.state.select(Some(index));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
