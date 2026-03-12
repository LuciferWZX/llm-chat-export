use std::sync::mpsc;
use std::thread;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use crate::i18n::{self, Lang, Texts};
use crate::platform;

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    LanguageSelect,
    PlatformSelect,
    TokenInput,
    Filter,
    Loading,
    ConversationList,
    Downloading,
    Done,
    Error,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PlatformKind {
    ChatGPT,
    ClaudeWeb,
    ClaudeCode,
    Cursor,
}

#[derive(Clone, Copy, PartialEq)]
pub enum KeywordMode {
    Include,
    Exclude,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FilterField {
    KeywordMode,
    Keywords,
    StartDate,
    EndDate,
}

pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub last_chat_time: String,
    pub selected: bool,
    pub project: String,
}

pub enum WorkerMsg {
    Conversations(Vec<Conversation>),
    DownloadProgress(usize),
    DownloadDone(String),
    Error(String),
}

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,

    // Language
    pub lang: Lang,
    pub lang_cursor: usize,

    pub platforms: Vec<(&'static str, &'static str, &'static str)>,
    pub platform_cursor: usize,
    pub platform: Option<PlatformKind>,

    pub token: String,
    pub token_visible: bool,

    // Filter
    pub filter_keyword_mode: KeywordMode,
    pub filter_keywords: String,
    pub filter_start_date: String,
    pub filter_end_date: String,
    pub filter_focus: FilterField,

    // Cached raw data
    all_conversations: Vec<Conversation>,
    pub has_fetched: bool,

    // Displayed (filtered) conversations
    pub conversations: Vec<Conversation>,
    pub list_state: ListState,

    pub download_current: usize,
    pub download_total: usize,
    pub download_path: String,

    pub error_msg: String,

    rx: Option<mpsc::Receiver<WorkerMsg>>,
}

impl App {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            screen: Screen::LanguageSelect,
            should_quit: false,
            lang: Lang::Zh,
            lang_cursor: 0,
            platforms: vec![
                ("ChatGPT", "OpenAI ChatGPT", "(Access Token)"),
                ("Claude Web", "Anthropic Claude.ai", "(Session Key)"),
                ("Claude Code", "", ""),
                ("Cursor", "", ""),
            ],
            platform_cursor: 0,
            platform: None,
            token: String::new(),
            token_visible: false,
            filter_keyword_mode: KeywordMode::Include,
            filter_keywords: String::new(),
            filter_start_date: String::new(),
            filter_end_date: String::new(),
            filter_focus: FilterField::Keywords,
            all_conversations: Vec::new(),
            has_fetched: false,
            conversations: Vec::new(),
            list_state,
            download_current: 0,
            download_total: 0,
            download_path: String::new(),
            error_msg: String::new(),
            rx: None,
        }
    }

    pub fn texts(&self) -> &'static Texts {
        i18n::texts(self.lang)
    }

    pub fn check_worker(&mut self) {
        let msg = match &self.rx {
            Some(rx) => match rx.try_recv() {
                Ok(msg) => Some(msg),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.rx = None;
                    None
                }
            },
            None => None,
        };

        if let Some(msg) = msg {
            match msg {
                WorkerMsg::Conversations(convs) => {
                    self.all_conversations = convs;
                    self.has_fetched = true;
                    self.rx = None;
                    self.apply_filters();
                    self.screen = Screen::ConversationList;
                }
                WorkerMsg::DownloadProgress(current) => {
                    self.download_current = current;
                }
                WorkerMsg::DownloadDone(path) => {
                    self.download_path = path;
                    self.screen = Screen::Done;
                    self.rx = None;
                }
                WorkerMsg::Error(e) => {
                    self.error_msg = e;
                    self.screen = Screen::Error;
                    self.rx = None;
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.screen {
            Screen::LanguageSelect => self.handle_language_select(key),
            Screen::PlatformSelect => self.handle_platform_select(key),
            Screen::TokenInput => self.handle_token_input(key),
            Screen::Filter => self.handle_filter(key),
            Screen::Loading => self.handle_loading(key),
            Screen::ConversationList => self.handle_conversation_list(key),
            Screen::Downloading => {}
            Screen::Done => self.handle_done(key),
            Screen::Error => self.handle_error(key),
        }
    }

    fn handle_language_select(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.lang_cursor > 0 {
                    self.lang_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.lang_cursor < 1 {
                    self.lang_cursor += 1;
                }
            }
            KeyCode::Enter => {
                self.lang = match self.lang_cursor {
                    0 => Lang::Zh,
                    _ => Lang::En,
                };
                self.screen = Screen::PlatformSelect;
            }
            _ => {}
        }
    }

    fn handle_platform_select(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => {
                self.screen = Screen::LanguageSelect;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.platform_cursor > 0 {
                    self.platform_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.platform_cursor < self.platforms.len() - 1 {
                    self.platform_cursor += 1;
                }
            }
            KeyCode::Enter => {
                let kind = match self.platform_cursor {
                    0 => PlatformKind::ChatGPT,
                    1 => PlatformKind::ClaudeWeb,
                    2 => PlatformKind::ClaudeCode,
                    _ => PlatformKind::Cursor,
                };
                self.platform = Some(kind);
                if kind == PlatformKind::ClaudeCode || kind == PlatformKind::Cursor {
                    self.screen = Screen::Filter;
                } else {
                    self.token.clear();
                    self.screen = Screen::TokenInput;
                }
            }
            _ => {}
        }
    }

    fn handle_token_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::PlatformSelect;
            }
            KeyCode::Enter => {
                if !self.token.is_empty() {
                    self.screen = Screen::Filter;
                }
            }
            KeyCode::Backspace => {
                self.token.pop();
            }
            KeyCode::Tab => {
                self.token_visible = !self.token_visible;
            }
            KeyCode::Char(c) => {
                self.token.push(c);
            }
            _ => {}
        }
    }

    fn handle_filter(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Go back to previous step
                self.all_conversations.clear();
                self.has_fetched = false;
                match self.platform {
                    Some(PlatformKind::ClaudeCode) | Some(PlatformKind::Cursor) => {
                        self.screen = Screen::PlatformSelect;
                    }
                    _ => {
                        self.screen = Screen::TokenInput;
                    }
                }
            }
            KeyCode::Up => {
                self.filter_focus = match self.filter_focus {
                    FilterField::KeywordMode => FilterField::EndDate,
                    FilterField::Keywords => FilterField::KeywordMode,
                    FilterField::StartDate => FilterField::Keywords,
                    FilterField::EndDate => FilterField::StartDate,
                };
            }
            KeyCode::Down | KeyCode::Tab => {
                self.filter_focus = match self.filter_focus {
                    FilterField::KeywordMode => FilterField::Keywords,
                    FilterField::Keywords => FilterField::StartDate,
                    FilterField::StartDate => FilterField::EndDate,
                    FilterField::EndDate => FilterField::KeywordMode,
                };
            }
            KeyCode::Char(' ') if self.filter_focus == FilterField::KeywordMode => {
                self.filter_keyword_mode = match self.filter_keyword_mode {
                    KeywordMode::Include => KeywordMode::Exclude,
                    KeywordMode::Exclude => KeywordMode::Include,
                };
            }
            KeyCode::Char(c) => match self.filter_focus {
                FilterField::Keywords => self.filter_keywords.push(c),
                FilterField::StartDate => {
                    if self.filter_start_date.len() < 10 {
                        self.filter_start_date.push(c);
                    }
                }
                FilterField::EndDate => {
                    if self.filter_end_date.len() < 10 {
                        self.filter_end_date.push(c);
                    }
                }
                _ => {}
            },
            KeyCode::Backspace => match self.filter_focus {
                FilterField::Keywords => {
                    self.filter_keywords.pop();
                }
                FilterField::StartDate => {
                    self.filter_start_date.pop();
                }
                FilterField::EndDate => {
                    self.filter_end_date.pop();
                }
                _ => {}
            },
            KeyCode::Enter => {
                if self.has_fetched {
                    self.apply_filters();
                    self.screen = Screen::ConversationList;
                } else {
                    self.start_loading();
                }
            }
            _ => {}
        }
    }

    fn handle_loading(&mut self, key: KeyEvent) {
        if let KeyCode::Esc = key.code {
            self.rx = None;
            self.screen = Screen::Filter;
        }
    }

    fn handle_conversation_list(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Esc => {
                // Go back to filter, keep cache for re-filtering
                self.screen = Screen::Filter;
            }
            KeyCode::Char('f') => {
                self.screen = Screen::Filter;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.list_state.selected().unwrap_or(0);
                if i > 0 {
                    self.list_state.select(Some(i - 1));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.list_state.selected().unwrap_or(0);
                if i < self.conversations.len().saturating_sub(1) {
                    self.list_state.select(Some(i + 1));
                }
            }
            KeyCode::Char(' ') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(conv) = self.conversations.get_mut(i) {
                        conv.selected = !conv.selected;
                    }
                }
            }
            KeyCode::Char('a') => {
                let all_selected = self.conversations.iter().all(|c| c.selected);
                for conv in &mut self.conversations {
                    conv.selected = !all_selected;
                }
            }
            KeyCode::Enter => {
                let has_selected = self.conversations.iter().any(|c| c.selected);
                if has_selected {
                    self.start_download();
                }
            }
            _ => {}
        }
    }

    fn handle_done(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Esc | KeyCode::Enter => {
                // Go back to conversation list for another export
                self.screen = Screen::ConversationList;
                self.download_current = 0;
                self.download_total = 0;
            }
            _ => {}
        }
    }

    fn handle_error(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                self.screen = Screen::Filter;
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn apply_filters(&mut self) {
        let keywords: Vec<String> = self
            .filter_keywords
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        self.conversations = self
            .all_conversations
            .iter()
            .filter(|conv| {
                // Keyword filter
                if !keywords.is_empty() {
                    let title_lower = conv.title.to_lowercase();
                    match self.filter_keyword_mode {
                        KeywordMode::Include => {
                            if !keywords.iter().any(|kw| title_lower.contains(kw.as_str())) {
                                return false;
                            }
                        }
                        KeywordMode::Exclude => {
                            if keywords.iter().any(|kw| title_lower.contains(kw.as_str())) {
                                return false;
                            }
                        }
                    }
                }

                // Date filter
                let date_source = if conv.last_chat_time.is_empty() {
                    &conv.created_at
                } else {
                    &conv.last_chat_time
                };
                let conv_date = date_source.get(..10).unwrap_or("");

                if !self.filter_start_date.is_empty()
                    && conv_date < self.filter_start_date.as_str()
                {
                    return false;
                }
                if !self.filter_end_date.is_empty() && conv_date > self.filter_end_date.as_str() {
                    return false;
                }

                true
            })
            .map(|c| Conversation {
                id: c.id.clone(),
                title: c.title.clone(),
                created_at: c.created_at.clone(),
                last_chat_time: c.last_chat_time.clone(),
                selected: false,
                project: c.project.clone(),
            })
            .collect();

        self.list_state = ListState::default();
        if !self.conversations.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn start_loading(&mut self) {
        self.screen = Screen::Loading;
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let platform = self.platform.unwrap();
        let token = self.token.clone();

        thread::spawn(move || {
            let result = match platform {
                PlatformKind::ChatGPT => platform::chatgpt::fetch_conversations(&token),
                PlatformKind::ClaudeWeb => platform::claude_web::fetch_conversations(&token),
                PlatformKind::ClaudeCode => platform::claude_code::fetch_conversations(),
                PlatformKind::Cursor => platform::cursor::fetch_conversations(),
            };
            match result {
                Ok(convs) => {
                    let _ = tx.send(WorkerMsg::Conversations(convs));
                }
                Err(e) => {
                    let _ = tx.send(WorkerMsg::Error(format!("{:#}", e)));
                }
            }
        });
    }

    fn start_download(&mut self) {
        self.download_current = 0;
        self.download_total = self.conversations.iter().filter(|c| c.selected).count();
        self.screen = Screen::Downloading;

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let platform = self.platform.unwrap();
        let token = self.token.clone();
        let selected: Vec<_> = self
            .conversations
            .iter()
            .filter(|c| c.selected)
            .map(|c| (c.id.clone(), c.title.clone(), c.project.clone()))
            .collect();

        let platform_name = match platform {
            PlatformKind::ChatGPT => "chatgpt",
            PlatformKind::ClaudeWeb => "claude-web",
            PlatformKind::ClaudeCode => "claude-code",
            PlatformKind::Cursor => "cursor",
        };

        thread::spawn(move || {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let export_dir = exe_dir
                .join("llm-exports")
                .join(platform_name)
                .to_string_lossy()
                .to_string();
            if let Err(e) = std::fs::create_dir_all(&export_dir) {
                let _ = tx.send(WorkerMsg::Error(format!("Create dir failed: {}", e)));
                return;
            }

            for (i, (id, title, project)) in selected.iter().enumerate() {
                let _ = tx.send(WorkerMsg::DownloadProgress(i + 1));

                let result = match platform {
                    PlatformKind::ChatGPT => {
                        platform::chatgpt::export_conversation(&token, id, title)
                    }
                    PlatformKind::ClaudeWeb => {
                        platform::claude_web::export_conversation(&token, id, title)
                    }
                    PlatformKind::ClaudeCode => {
                        platform::claude_code::export_conversation(id, title, project)
                    }
                    PlatformKind::Cursor => {
                        platform::cursor::export_conversation(id, title)
                    }
                };

                match result {
                    Ok(content) => {
                        let safe = sanitize_filename(title);
                        let filename = if safe.is_empty() {
                            format!("{}.md", &id[..8.min(id.len())])
                        } else {
                            format!("{}.md", safe)
                        };
                        let path = format!("{}/{}", export_dir, filename);
                        if let Err(e) = std::fs::write(&path, &content) {
                            let _ =
                                tx.send(WorkerMsg::Error(format!("Write failed {}: {}", path, e)));
                            return;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(WorkerMsg::Error(format!(
                            "Export '{}' failed: {:#}",
                            title, e
                        )));
                        return;
                    }
                }
            }

            let _ = tx.send(WorkerMsg::DownloadDone(export_dir));
        });
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\n' | '\r' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .take(100)
        .collect::<String>()
        .trim()
        .to_string()
}
