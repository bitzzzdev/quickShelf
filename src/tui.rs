use crate::config::{ShelfConfig, TuiLayout, TuiStyle};
use crate::images::ImageManager;
use crate::ipc::{IpcMessage, start_server};
use crate::shortcuts::{Shortcut, ShortcutManager};
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
use std::collections::HashMap;
use std::io::{self, Stdout};
use std::path::Path;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

#[derive(Clone)]
enum ShelfItem {
    Shortcut(Shortcut),
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
    content_area: Rect,
    visible: bool,
    message: String,
    thumbnail_cache: HashMap<String, Vec<Vec<(char, Color, Color)>>>,
    app_picker_open: bool,
    app_picker_search: String,
    app_picker_selected: usize,
    system_apps: Vec<Shortcut>,
    grid_cols: usize,
    card_width: u16,
    card_height: u16,
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
            content_area: Rect::default(),
            visible: true,
            message: "Ready".to_string(),
            thumbnail_cache: HashMap::new(),
            app_picker_open: false,
            app_picker_search: String::new(),
            app_picker_selected: 0,
            system_apps: Vec::new(),
            grid_cols: 2,
            card_width: 28,
            card_height: 7,
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
        self.thumbnail_cache.clear();
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

    fn filtered_system_apps(&self) -> Vec<Shortcut> {
        if self.app_picker_search.is_empty() {
            return self.system_apps.clone();
        }
        let query = self.app_picker_search.to_lowercase();
        self.system_apps
            .iter()
            .filter(|app| {
                app.name.to_lowercase().contains(&query) || app.exec.to_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        if self.app_picker_open {
            match code {
                KeyCode::Esc => {
                    self.app_picker_open = false;
                    self.message = "App picker closed".into();
                }
                KeyCode::Char(c) => {
                    self.app_picker_search.push(c);
                    self.app_picker_selected = 0;
                }
                KeyCode::Backspace => {
                    self.app_picker_search.pop();
                    self.app_picker_selected = 0;
                }
                KeyCode::Down | KeyCode::Tab => {
                    let count = self.filtered_system_apps().len();
                    self.app_picker_selected =
                        (self.app_picker_selected + 1).min(count.saturating_sub(1));
                }
                KeyCode::Up | KeyCode::BackTab => {
                    self.app_picker_selected = self.app_picker_selected.saturating_sub(1);
                }
                KeyCode::Enter => {
                    let matching = self.filtered_system_apps();
                    if let Some(app) = matching.get(self.app_picker_selected) {
                        self.shortcuts.add(app.clone());
                        let _ = self.shortcuts.save();
                        self.app_picker_open = false;
                        self.reload();
                        self.message = format!("Added shortcut: {}", app.name);
                    }
                }
                _ => {}
            }
            return false;
        }

        let len = self.items().len();
        let is_bento = self.config.tui.layout.view_mode == "bento";
        let step = if is_bento { self.grid_cols.max(1) } else { 1 };

        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('v') | KeyCode::Char('m') => {
                let new_mode = if self.config.tui.layout.view_mode == "bento" {
                    "list".to_string()
                } else {
                    "bento".to_string()
                };
                self.config.tui.layout.view_mode = new_mode.clone();
                let _ = self.config.save();
                self.message = format!("View format switched to {}", new_mode);
            }
            KeyCode::Char('a') | KeyCode::Char('s') => {
                self.app_picker_open = true;
                self.app_picker_search.clear();
                self.app_picker_selected = 0;
                self.system_apps = ShortcutManager::list_system_apps();
                self.message = "Search installed applications and press Enter to add".into();
            }
            KeyCode::Char('x') | KeyCode::Char('w') | KeyCode::Char('y') => {
                self.trigger_external_drag(self.selected);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + step).min(len.saturating_sub(1));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(step);
            }
            KeyCode::Right | KeyCode::Char('l') if is_bento => {
                self.selected = (self.selected + 1).min(len.saturating_sub(1));
            }
            KeyCode::Left | KeyCode::Char('h') if is_bento => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.activate_selected(),
            KeyCode::Delete | KeyCode::Char('d') => self.remove_selected(),
            KeyCode::Char('r') => self.reload(),
            _ => {}
        }
        false
    }

    fn trigger_external_drag(&mut self, index: usize) {
        if let Some(item) = self.items().get(index) {
            match item {
                ShelfItem::Shortcut(sc) => {
                    let icon_path = ShortcutManager::find_icon_path(&sc.icon);
                    let path = icon_path.as_deref().unwrap_or_else(|| Path::new(&sc.exec));
                    self.message = ImageManager::start_external_drag(path, false);
                }
                ShelfItem::Image(img) => {
                    self.message = ImageManager::start_external_drag(Path::new(&img.path), true);
                }
            }
        }
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
        if self.app_picker_open {
            match kind {
                MouseEventKind::ScrollDown => {
                    let count = self.filtered_system_apps().len();
                    self.app_picker_selected =
                        (self.app_picker_selected + 1).min(count.saturating_sub(1));
                }
                MouseEventKind::ScrollUp => {
                    self.app_picker_selected = self.app_picker_selected.saturating_sub(1);
                }
                _ => {}
            }
            return;
        }

        let is_outside = column < self.content_area.x
            || column >= self.content_area.x + self.content_area.width
            || row < self.content_area.y
            || row >= self.content_area.y + self.content_area.height;

        if is_outside && matches!(kind, MouseEventKind::Drag(MouseButton::Left)) {
            if let Some(from) = self.dragging.take() {
                self.trigger_external_drag(from);
                return;
            }
        }

        let Some(index) = self.item_at(column, row) else { return };
        match kind {
            MouseEventKind::Down(MouseButton::Middle) => {
                self.selected = index;
                self.trigger_external_drag(index);
            }
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
            MouseEventKind::ScrollDown => {
                self.selected = (self.selected + 1).min(self.items().len().saturating_sub(1))
            }
            MouseEventKind::ScrollUp => self.selected = self.selected.saturating_sub(1),
            _ => {}
        }
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

    fn item_at(&self, column: u16, row: u16) -> Option<usize> {
        let len = self.items().len();
        if self.config.tui.layout.view_mode == "bento" {
            if row < self.content_area.y
                || row >= self.content_area.y + self.content_area.height
                || column < self.content_area.x
                || column >= self.content_area.x + self.content_area.width
            {
                return None;
            }
            let col_idx = ((column - self.content_area.x) / self.card_width.max(1)) as usize;
            let row_idx = ((row - self.content_area.y) / self.card_height.max(1)) as usize;
            let index = row_idx * self.grid_cols + col_idx;
            (index < len).then_some(index)
        } else {
            let index = row
                .checked_sub(self.content_top.saturating_add(1))
                .map(|value| value as usize)?;
            (index < len).then_some(index)
        }
    }

    fn get_cached_thumbnail(&mut self, key: &str, path: &Path, cols: u16, rows: u16) -> Vec<Vec<(char, Color, Color)>> {
        if let Some(cached) = self.thumbnail_cache.get(key) {
            return cached.clone();
        }
        let generated = ImageManager::generate_halfblock_thumbnail(path, cols, rows);
        self.thumbnail_cache.insert(key.to_string(), generated.clone());
        generated
    }

    fn render(&mut self, frame: &mut Frame) {
        if !self.visible {
            frame.render_widget(Clear, frame.area());
            return;
        }

        let layout = self.config.tui.layout.clone();
        let area = compute_shelf_rect(&layout, frame.area());
        let colors = Palette::from(&self.config.tui);

        let view_badge = if layout.view_mode == "bento" {
            " [Bento] "
        } else {
            " [List] "
        };

        let outer = Block::default()
            .title(Line::from(vec![
                Span::styled(" 󱂬 ", Style::default().fg(colors.accent)),
                Span::styled(
                    format!("{} ", layout.title),
                    Style::default().fg(colors.foreground).add_modifier(Modifier::BOLD),
                ),
                Span::styled(view_badge, Style::default().fg(colors.accent)),
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
        self.content_area = Rect {
            x: chunks[1].x.saturating_add(padding),
            y: self.content_top,
            width: chunks[1].width.saturating_sub(padding.saturating_mul(2)),
            height: chunks[1].height.saturating_sub(padding.saturating_mul(2)),
        };

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

        if self.app_picker_open {
            self.render_app_picker(frame, chunks[1], &colors);
        } else if layout.view_mode == "bento" {
            self.render_bento_grid(frame, self.content_area, &colors);
        } else {
            self.render_list_view(frame, self.content_area, &colors);
        }

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

    fn render_bento_grid(&mut self, frame: &mut Frame, area: Rect, colors: &Palette) {
        let items = self.items();
        if items.is_empty() {
            let empty = Paragraph::new(Line::from(Span::styled(
                "Shelf is empty. Press 'a' to add apps, or drop images here.",
                Style::default().fg(colors.muted),
            )))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(colors.card)));
            frame.render_widget(empty, area);
            return;
        }

        let cols = if area.width >= 82 { 3 } else { 2 };
        self.grid_cols = cols;
        let card_w = area.width / cols as u16;
        let card_h = 7u16;
        self.card_width = card_w;
        self.card_height = card_h;

        let max_visible_cards = (cols as u16 * (area.height / card_h)) as usize;
        let scroll_offset = if self.selected >= max_visible_cards {
            (self.selected / cols) * cols
        } else {
            0
        };

        for (slot, item) in items.iter().enumerate().skip(scroll_offset).take(max_visible_cards) {
            let visible_idx = slot - scroll_offset;
            let col = (visible_idx % cols) as u16;
            let row = (visible_idx / cols) as u16;

            let card_rect = Rect {
                x: area.x + col * card_w,
                y: area.y + row * card_h,
                width: card_w.saturating_sub(1),
                height: card_h,
            };

            let is_sel = slot == self.selected;
            let border_color = if is_sel { colors.accent } else { colors.card };
            let bg_color = if is_sel { colors.selected } else { colors.card };

            let (title, detail, nerd_icon, thumb_data) = match item {
                ShelfItem::Shortcut(sc) => {
                    let key = format!("app:{}:10x2", sc.id);
                    let icon_path = ShortcutManager::find_icon_path(&sc.icon);
                    let thumb = if let Some(path) = icon_path {
                        self.get_cached_thumbnail(&key, &path, 6, 2)
                    } else {
                        Vec::new()
                    };
                    (sc.name.clone(), sc.exec.clone(), ShortcutManager::nerd_font_icon(&sc.icon, &sc.exec), thumb)
                }
                ShelfItem::Image(img) => {
                    let key = format!("img:{}:10x2", img.id);
                    let thumb = self.get_cached_thumbnail(&key, Path::new(&img.path), 8, 2);
                    (img.name.clone(), format!("{}×{} px", img.width, img.height), "󰋩", thumb)
                }
            };

            let card_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(bg_color));
            frame.render_widget(card_block, card_rect);

            let inner_x = card_rect.x.saturating_add(1);
            let inner_y = card_rect.y.saturating_add(1);
            let inner_w = card_rect.width.saturating_sub(2);

            if !thumb_data.is_empty() {
                for (r_idx, line) in thumb_data.iter().take(2).enumerate() {
                    let line_y = inner_y + r_idx as u16;
                    if line_y < card_rect.y + card_rect.height.saturating_sub(1) {
                        let spans = line.iter().map(|(ch, fg, bg)| {
                            Span::styled(ch.to_string(), Style::default().fg(*fg).bg(*bg))
                        }).collect::<Vec<_>>();
                        frame.render_widget(Paragraph::new(Line::from(spans)), Rect {
                            x: inner_x,
                            y: line_y,
                            width: inner_w.min(line.len() as u16),
                            height: 1,
                        });
                    }
                }
            } else {
                let badge = Line::from(vec![
                    Span::styled(format!(" {} ", nerd_icon), Style::default().fg(colors.accent).add_modifier(Modifier::BOLD)),
                ]);
                frame.render_widget(Paragraph::new(badge), Rect {
                    x: inner_x,
                    y: inner_y,
                    width: inner_w,
                    height: 1,
                });
            }

            let text_y = inner_y + 2;
            if text_y < card_rect.y + card_rect.height {
                let title_style = if is_sel {
                    Style::default().fg(colors.foreground).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(colors.foreground)
                };
                let title_line = Line::from(Span::styled(format!("{:<width$}", title, width = inner_w as usize), title_style));
                frame.render_widget(Paragraph::new(title_line), Rect {
                    x: inner_x,
                    y: text_y,
                    width: inner_w,
                    height: 1,
                });
            }

            let detail_y = inner_y + 3;
            if detail_y < card_rect.y + card_rect.height {
                let detail_line = Line::from(Span::styled(format!("{:<width$}", detail, width = inner_w as usize), Style::default().fg(colors.muted)));
                frame.render_widget(Paragraph::new(detail_line), Rect {
                    x: inner_x,
                    y: detail_y,
                    width: inner_w,
                    height: 1,
                });
            }
        }
    }

    fn render_list_view(&mut self, frame: &mut Frame, area: Rect, colors: &Palette) {
        let items = self.items();
        let layout = &self.config.tui.layout;

        let list_items = items.iter().enumerate().map(|(index, item)| {
            let (nerd_icon, name, detail) = match item {
                ShelfItem::Shortcut(shortcut) => {
                    (
                        ShortcutManager::nerd_font_icon(&shortcut.icon, &shortcut.exec),
                        shortcut.name.clone(),
                        shortcut.exec.clone(),
                    )
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
                Span::styled(format!(" {} ", nerd_icon), Style::default().fg(colors.accent)),
                Span::styled(format!("{:<26}", name), style),
            ];
            if layout.show_details {
                line.push(Span::styled(
                    format!("  {:<width$}", detail, width = layout.detail_width as usize),
                    Style::default().fg(colors.muted),
                ));
            }
            ListItem::new(Line::from(line))
        }).collect::<Vec<_>>();

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::LEFT | Borders::RIGHT).border_style(Style::default().fg(colors.card)));
        frame.render_widget(list, area);
    }

    fn render_app_picker(&self, frame: &mut Frame, area: Rect, colors: &Palette) {
        let modal_area = centered_rect(80, 80, area);
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .title(Line::from(vec![
                Span::styled(" 󰣆 Add Application Shortcut ", Style::default().fg(colors.accent).add_modifier(Modifier::BOLD)),
                Span::styled("(Type to search, Enter to add, Esc to cancel) ", Style::default().fg(colors.muted)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.accent))
            .style(Style::default().bg(colors.background));
        frame.render_widget(block, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(Rect {
                x: modal_area.x + 1,
                y: modal_area.y + 1,
                width: modal_area.width.saturating_sub(2),
                height: modal_area.height.saturating_sub(2),
            });

        let search_bar = Paragraph::new(Line::from(vec![
            Span::styled(" Search: ", Style::default().fg(colors.accent)),
            Span::styled(&self.app_picker_search, Style::default().fg(colors.foreground).add_modifier(Modifier::BOLD)),
            Span::styled(" █", Style::default().fg(colors.muted)),
        ]))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(colors.card)));
        frame.render_widget(search_bar, chunks[0]);

        let matching = self.filtered_system_apps();
        let items = matching.iter().enumerate().map(|(idx, app)| {
            let is_sel = idx == self.app_picker_selected;
            let icon = ShortcutManager::nerd_font_icon(&app.icon, &app.exec);
            let style = if is_sel {
                Style::default().fg(colors.foreground).bg(colors.selected).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.foreground)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(colors.accent)),
                Span::styled(format!("{:<28}", app.name), style),
                Span::styled(format!(" {}", app.exec), Style::default().fg(colors.muted)),
            ]))
        }).collect::<Vec<_>>();

        let list = List::new(items);
        frame.render_widget(list, chunks[1]);
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

fn compute_shelf_rect(layout: &TuiLayout, area: Rect) -> Rect {
    match layout.aspect_ratio.as_str() {
        "1:1" => {
            let max_h = area.height.saturating_sub(2);
            let max_w = area.width.saturating_sub(2);
            let target_h = (max_h as f32).min(max_w as f32 / 2.1) as u16;
            let target_w = ((target_h as f32) * 2.1) as u16;
            let target_w = target_w.clamp(24, max_w);
            let target_h = target_h.clamp(12, max_h);
            let x = area.x + (area.width.saturating_sub(target_w)) / 2;
            let y = area.y + (area.height.saturating_sub(target_h)) / 2;
            Rect {
                x,
                y,
                width: target_w,
                height: target_h,
            }
        }
        "16:9" => {
            let max_h = area.height.saturating_sub(2);
            let max_w = area.width.saturating_sub(2);
            let target_h = (max_h as f32).min((max_w as f32 * 9.0) / (16.0 * 2.1)) as u16;
            let target_w = ((target_h as f32 * 16.0 * 2.1) / 9.0) as u16;
            let target_w = target_w.clamp(24, max_w);
            let target_h = target_h.clamp(12, max_h);
            let x = area.x + (area.width.saturating_sub(target_w)) / 2;
            let y = area.y + (area.height.saturating_sub(target_h)) / 2;
            Rect { x, y, width: target_w, height: target_h }
        }
        "4:3" => {
            let max_h = area.height.saturating_sub(2);
            let max_w = area.width.saturating_sub(2);
            let target_h = (max_h as f32).min((max_w as f32 * 3.0) / (4.0 * 2.1)) as u16;
            let target_w = ((target_h as f32 * 4.0 * 2.1) / 3.0) as u16;
            let target_w = target_w.clamp(24, max_w);
            let target_h = target_h.clamp(12, max_h);
            let x = area.x + (area.width.saturating_sub(target_w)) / 2;
            let y = area.y + (area.height.saturating_sub(target_h)) / 2;
            Rect { x, y, width: target_w, height: target_h }
        }
        _ => centered_rect(
            layout.width_percent.clamp(30, 100),
            layout.height_percent.clamp(30, 100),
            area,
        ),
    }
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
