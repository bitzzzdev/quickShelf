use crate::config::{ShelfConfig, TuiStyle};
use crate::images::ImageManager;
use crate::ipc::{IpcMessage, start_server};
use crate::shortcuts::ShortcutManager;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste,
        EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use std::io::{self, Stdout};
use std::path::Path;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

#[derive(Clone)]
enum ShelfItem {
    Shortcut(crate::shortcuts::Shortcut),
    Image(crate::images::ImageItem),
}

pub fn run_tui() -> io::Result<()> {
    let (sender, receiver) = channel::<IpcMessage>();
    start_server(sender)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = TuiApp::new(receiver).run(&mut terminal);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    )?;
    terminal.show_cursor()?;
    result
}

struct TuiApp {
    config: ShelfConfig,
    shortcuts: ShortcutManager,
    images: ImageManager,
    receiver: Receiver<IpcMessage>,
    selected: usize,
    dragging: Option<usize>,
    drag_target: Option<usize>,
    content_top: u16,
    visible: bool,
    message: String,
}

impl TuiApp {
    fn new(receiver: Receiver<IpcMessage>) -> Self {
        Self {
            config: ShelfConfig::load(),
            shortcuts: ShortcutManager::load(),
            images: ImageManager::load(),
            receiver,
            selected: 0,
            dragging: None,
            drag_target: None,
            content_top: 0,
            visible: true,
            message: "Ready".to_string(),
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        loop {
            while let Ok(message) = self.receiver.try_recv() {
                match message {
                    IpcMessage::Toggle => self.visible = !self.visible,
                    IpcMessage::Show => self.visible = true,
                    IpcMessage::Hide => self.visible = false,
                    IpcMessage::ReloadStyle | IpcMessage::SetTheme(_) | IpcMessage::Refresh => {
                        self.reload()
                    }
                    IpcMessage::AddImage(path) => {
                        let _ = self.images.add_from_path(Path::new(&path));
                        self.reload();
                    }
                    IpcMessage::AddShortcut(path) => {
                        if let Some(shortcut) =
                            ShortcutManager::from_desktop_file(Path::new(&path))
                        {
                            self.shortcuts.add(shortcut);
                        }
                        self.reload();
                    }
                }
            }

            terminal.draw(|frame| self.render(frame))?;
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if self.handle_key(key.code) {
                            break;
                        }
                    }
                    Event::Mouse(mouse) => self.handle_mouse(mouse.kind, mouse.column, mouse.row),
                    Event::Paste(text) => self.add_dropped_paths(&text),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn reload(&mut self) {
        self.config = ShelfConfig::load();
        self.shortcuts = ShortcutManager::load();
        self.images = ImageManager::load();
        self.selected = self.selected.min(self.items().len().saturating_sub(1));
    }

    fn items(&self) -> Vec<ShelfItem> {
        let apps = || {
            self.shortcuts
                .list()
                .iter()
                .cloned()
                .map(ShelfItem::Shortcut)
                .collect::<Vec<_>>()
        };
        let images = || {
            self.images
                .list()
                .iter()
                .cloned()
                .map(ShelfItem::Image)
                .collect::<Vec<_>>()
        };
        let mut items = Vec::new();
        for section in &self.config.tui.layout.section_order {
            match section.as_str() {
                "apps" | "shortcuts" => items.extend(apps()),
                "images" | "media" => items.extend(images()),
                _ => {}
            }
        }
        if !self
            .config
            .tui
            .layout
            .section_order
            .iter()
            .any(|section| matches!(section.as_str(), "apps" | "shortcuts"))
        {
            items.extend(apps());
        }
        if !self
            .config
            .tui
            .layout
            .section_order
            .iter()
            .any(|section| matches!(section.as_str(), "images" | "media"))
        {
            items.extend(images());
        }
        items
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        let len = self.items().len();
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1).min(len.saturating_sub(1))
            }
            KeyCode::Up | KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
            KeyCode::Enter | KeyCode::Char(' ') => self.activate_selected(),
            KeyCode::Delete | KeyCode::Char('d') => self.remove_selected(),
            KeyCode::Char('r') => self.reload(),
            KeyCode::Char('a') => self.message = "Drop a .desktop path or image path here".into(),
            _ => {}
        }
        false
    }

    fn activate_selected(&mut self) {
        if let Some(item) = self.items().get(self.selected) {
            match item {
                ShelfItem::Shortcut(shortcut) => {
                    match ShortcutManager::launch_exec(&shortcut.exec) {
                        Ok(()) => self.message = format!("Launched {}", shortcut.name),
                        Err(error) => self.message = format!("Launch failed: {}", error),
                    }
                }
                ShelfItem::Image(image) => {
                    ImageManager::open_image(Path::new(&image.path));
                    self.message = format!("Opened {}", image.name);
                }
            }
        }
    }

    fn remove_selected(&mut self) {
        if let Some(item) = self.items().get(self.selected).cloned() {
            match item {
                ShelfItem::Shortcut(shortcut) => {
                    self.shortcuts.remove(&shortcut.id);
                    self.message = format!("Removed {}", shortcut.name);
                }
                ShelfItem::Image(image) => {
                    self.images.remove(&image.id);
                    self.message = format!("Removed {}", image.name);
                }
            }
            self.reload();
        }
    }

    fn add_dropped_paths(&mut self, text: &str) {
        let mut changed = false;
        for raw in text.split_whitespace() {
            let path = raw
                .trim()
                .trim_start_matches("file://")
                .trim_matches(|character| character == '"' || character == '\'');
            let path = Path::new(path);
            if path.extension().and_then(|extension| extension.to_str()) == Some("desktop") {
                if let Some(shortcut) = ShortcutManager::from_desktop_file(path) {
                    changed |= self.shortcuts.add(shortcut);
                }
            } else if ImageManager::is_image_path(path) {
                changed |= self.images.add_from_path(path).is_ok();
            }
        }
        if changed {
            self.reload();
            self.message = "Added dropped item".into();
        }
    }

    fn handle_mouse(&mut self, kind: MouseEventKind, column: u16, row: u16) {
        let Some(index) = self.item_at(row) else { return };
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.selected = index;
                self.dragging = Some(index);
                self.drag_target = Some(index);
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.selected = index;
                self.drag_target = Some(index);
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if let Some(from) = self.dragging.take() {
                    let to = self.drag_target.take().unwrap_or(index);
                    if from != to {
                        self.reorder(from, to);
                    } else {
                        self.activate_selected();
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.selected = index;
                self.remove_selected();
            }
            MouseEventKind::ScrollDown => self.selected = (self.selected + 1).min(self.items().len().saturating_sub(1)),
            MouseEventKind::ScrollUp => self.selected = self.selected.saturating_sub(1),
            _ => {}
        }
        let _ = column;
    }

    fn reorder(&mut self, from: usize, to: usize) {
        let items = self.items();
        let Some((from_is_app, from_position)) = self.item_position(&items, from) else {
            return;
        };
        let Some((to_is_app, to_position)) = self.item_position(&items, to) else {
            return;
        };
        if from_is_app != to_is_app {
            self.message = "Apps and images stay in separate shelves".into();
            return;
        }
        if from_is_app {
            self.shortcuts.move_item(from_position, to_position);
            self.message = "Reordered shortcut".into();
        } else {
            self.images.move_item(from_position, to_position);
            self.message = "Reordered image".into();
        }
        self.selected = to;
    }

    fn item_position(&self, items: &[ShelfItem], index: usize) -> Option<(bool, usize)> {
        let item = items.get(index)?;
        let is_app = matches!(item, ShelfItem::Shortcut(_));
        let position = items[..index]
            .iter()
            .filter(|item| matches!(item, ShelfItem::Shortcut(_)) == is_app)
            .count();
        Some((is_app, position))
    }

    fn item_at(&self, row: u16) -> Option<usize> {
        let index = row
            .checked_sub(self.content_top.saturating_add(1))
            .map(|value| value as usize)?;
        (index < self.items().len()).then_some(index)
    }

    fn render(&mut self, frame: &mut Frame) {
        if !self.visible {
            frame.render_widget(Clear, frame.area());
            return;
        }

        let layout = &self.config.tui.layout;
        let area = centered_rect(
            layout.width_percent.clamp(30, 100),
            layout.height_percent.clamp(30, 100),
            frame.area(),
        );
        let colors = Palette::from(&self.config.tui);
        let outer = Block::default()
            .title(Line::from(vec![
                Span::styled(" 󱂬 ", Style::default().fg(colors.accent)),
                Span::styled(
                    format!("{} ", layout.title),
                    Style::default().fg(colors.foreground).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} items ", self.items().len()),
                    Style::default().fg(colors.muted),
                ),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent))
            .style(Style::default().bg(colors.background).fg(colors.foreground));
        let outer = if layout.show_footer {
            outer.title_bottom(Line::from(Span::styled(
                format!(" {} ", layout.footer_hint),
                Style::default().fg(colors.muted),
            )))
        } else {
            outer
        };
        frame.render_widget(outer, area);

        let header_height = if layout.show_header {
            layout.header_height.max(1)
        } else {
            0
        };
        let footer_height = if layout.show_footer {
            layout.footer_height.max(1)
        } else {
            0
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height),
                Constraint::Min(1),
                Constraint::Length(footer_height),
            ])
            .split(area);
        let padding = layout.content_padding.min(chunks[1].height.saturating_sub(1));
        self.content_top = chunks[1].y.saturating_add(padding);
        if layout.show_header {
            let search = Paragraph::new(Line::from(vec![
                Span::styled("  / ", Style::default().fg(colors.accent)),
                Span::styled(&layout.subtitle, Style::default().fg(colors.muted)),
            ]))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(colors.card)),
            );
            frame.render_widget(search, chunks[0]);
        }

        let items = self.items();
        let list = List::new(items.iter().enumerate().map(|(index, item)| {
            let (icon, name, detail) = match item {
                ShelfItem::Shortcut(shortcut) => {
                    ("󰣆", shortcut.name.clone(), shortcut.exec.clone())
                }
                ShelfItem::Image(image) => (
                    "󰋩",
                    image.name.clone(),
                    format!("{} × {} px", image.width, image.height),
                ),
            };
            let style = if index == self.selected {
                Style::default().fg(colors.foreground).bg(colors.selected).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.foreground)
            };
            let mut line = vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(colors.accent)),
                Span::styled(format!("{:<24}", name), style),
            ];
            if layout.show_details {
                line.push(Span::styled(
                    format!("  {:<width$}", detail, width = layout.detail_width as usize),
                    Style::default().fg(colors.muted),
                ));
            }
            ListItem::new(Line::from(line))
        }))
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT).border_style(Style::default().fg(colors.card)));
        let list_area = Rect {
            x: chunks[1].x.saturating_add(padding),
            y: self.content_top,
            width: chunks[1].width.saturating_sub(padding.saturating_mul(2)),
            height: chunks[1].height.saturating_sub(padding.saturating_mul(2)),
        };
        frame.render_widget(list, list_area);

        if layout.show_footer {
            let footer = Paragraph::new(Text::from(vec![
                Line::from(Span::styled(
                    format!(" {}", self.message),
                    Style::default().fg(colors.muted),
                )),
            ]))
            .wrap(Wrap { trim: true });
            frame.render_widget(footer, chunks[2]);
        }
    }
}

struct Palette {
    accent: Color,
    background: Color,
    foreground: Color,
    muted: Color,
    selected: Color,
    card: Color,
}

impl From<&TuiStyle> for Palette {
    fn from(style: &TuiStyle) -> Self {
        Self {
            accent: parse_color(&style.accent),
            background: parse_color(&style.background),
            foreground: parse_color(&style.foreground),
            muted: parse_color(&style.muted),
            selected: parse_color(&style.selected),
            card: parse_color(&style.card),
        }
    }
}

fn parse_color(value: &str) -> Color {
    let value = value.trim();
    let value = value.trim_start_matches('#');
    let hex = value.get(..6).filter(|_| value.len() == 6 || value.len() == 8);
    if let Some(value) = hex {
        let red = u8::from_str_radix(&value[0..2], 16);
        let green = u8::from_str_radix(&value[2..4], 16);
        let blue = u8::from_str_radix(&value[4..6], 16);
        if let (Ok(red), Ok(green), Ok(blue)) = (red, green, blue) {
            return Color::Rgb(red, green, blue);
        }
    }
    let function = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))
        .and_then(|value| value.strip_suffix(')'));
    if let Some(value) = function {
        let channels = value
            .split(',')
            .take(3)
            .map(str::trim)
            .map(str::parse::<u8>)
            .collect::<Result<Vec<_>, _>>();
        if let Ok(channels) = channels {
            if channels.len() == 3 {
                return Color::Rgb(channels[0], channels[1], channels[2]);
            }
        }
    }
    Color::Reset
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
