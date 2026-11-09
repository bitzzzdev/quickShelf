use crate::config::ShelfConfig;
use crate::images::{ImageItem, ImageManager};
use crate::shortcuts::ShortcutManager;
use crate::themes;
use gdk::prelude::*;
use gdk_pixbuf::Pixbuf;
use gtk::pango;
use gtk::prelude::*;
use gtk_layer_shell::LayerShell;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct AppState {
    pub shortcuts: ShortcutManager,
    pub images: ImageManager,
    pub config: ShelfConfig,
    pub active_view: String, // "both", "apps", "media"
}

#[derive(Clone)]
pub struct MainWindow {
    window: gtk::Window,
    state: Rc<RefCell<AppState>>,
    content_box: gtk::Box,
    preview_box: gtk::Box,
    main_stack: gtk::Stack,
    css_provider: gtk::CssProvider,
}

impl MainWindow {
    pub fn new(is_homescreen: bool) -> Self {
        let config = ShelfConfig::load();
        let shortcuts = ShortcutManager::load();
        let images = ImageManager::load();
        let active_view = config.default_view.clone();

        let state = Rc::new(RefCell::new(AppState {
            shortcuts,
            images,
            config: config.clone(),
            active_view,
        }));

        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_title("Quick Shelf");
        window.set_role("quick-shelf");
        window.set_default_size(config.width, -1);
        window.set_resizable(!is_homescreen);

        if is_homescreen && gtk_layer_shell::is_supported() {
            window.init_layer_shell();
            let layer = if config.layer.to_lowercase() == "background" {
                gtk_layer_shell::Layer::Background
            } else {
                gtk_layer_shell::Layer::Bottom
            };
            window.set_layer(layer);
            window.set_keyboard_mode(gtk_layer_shell::KeyboardMode::OnDemand);
            window.set_exclusive_zone(0);

            // Anchors
            let anchor = config.homescreen_anchor.to_lowercase();
            if anchor.contains("top") {
                window.set_anchor(gtk_layer_shell::Edge::Top, true);
                window.set_layer_shell_margin(gtk_layer_shell::Edge::Top, config.margin_top);
            }
            if anchor.contains("bottom") {
                window.set_anchor(gtk_layer_shell::Edge::Bottom, true);
                window.set_layer_shell_margin(gtk_layer_shell::Edge::Bottom, config.margin_bottom);
            }
            if anchor.contains("right") {
                window.set_anchor(gtk_layer_shell::Edge::Right, true);
                window.set_layer_shell_margin(gtk_layer_shell::Edge::Right, config.margin_right);
            }
            if anchor.contains("left") {
                window.set_anchor(gtk_layer_shell::Edge::Left, true);
                window.set_layer_shell_margin(gtk_layer_shell::Edge::Left, config.margin_left);
            }
        } else {
            // Popup mode must remain a normal compositor window so Hyprland
            // can float, focus, resize, and move it with SUPER + mouse.
            window.set_type_hint(gdk::WindowTypeHint::Utility);
            window.set_skip_taskbar_hint(true);
            window.set_decorated(false);
            if is_homescreen {
                window.set_keep_below(true);
            } else {
                window.set_keep_above(true);
            }
        }

        // Apply CSS
        let css_provider = gtk::CssProvider::new();
        Self::load_style(&css_provider, &config.theme);
        if let Some(screen) = gdk::Screen::default() {
            gtk::StyleContext::add_provider_for_screen(
                &screen,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let outer_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer_box.style_context().add_class("quick-shelf-window");

        let main_stack = gtk::Stack::new();
        main_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        main_stack.set_transition_duration(150);

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        content_box.style_context().add_class("qs-mainbox");
        let preview_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        preview_box.style_context().add_class("qs-preview-overlay");

        main_stack.add_named(&content_box, "content");
        main_stack.add_named(&preview_box, "preview");

        outer_box.pack_start(&main_stack, true, true, 0);
        window.add(&outer_box);

        let app = Self {
            window,
            state,
            content_box,
            preview_box,
            main_stack,
            css_provider,
        };

        app.setup_drag_and_drop();
        app.render();

        app
    }

    #[allow(dead_code)]
    pub fn window(&self) -> &gtk::Window {
        &self.window
    }

    pub fn toggle_visibility(&self) {
        if self.window.is_visible() {
            self.window.hide();
        } else {
            self.refresh();
            self.window.show_all();
            self.window.present();
        }
    }

    pub fn show(&self) {
        self.refresh();
        self.window.show_all();
        self.window.present();
    }

    pub fn hide(&self) {
        self.window.hide();
    }

    pub fn reload_style(&self) {
        let theme = self.state.borrow().config.theme.clone();
        Self::load_style(&self.css_provider, &theme);
    }

    pub fn set_theme(&self, theme: &str) {
        {
            let mut st = self.state.borrow_mut();
            st.config.theme = theme.to_string();
            let _ = st.config.save();
        }
        self.reload_style();
    }

    fn load_style(provider: &gtk::CssProvider, theme: &str) {
        let css = themes::load_effective_css(theme);
        let _ = provider.load_from_data(css.as_bytes());
    }

    pub fn refresh(&self) {
        {
            let mut st = self.state.borrow_mut();
            st.shortcuts = ShortcutManager::load();
            st.images = ImageManager::load();
        }
        self.render();
    }

    fn setup_drag_and_drop(&self) {
        let targets = [gtk::TargetEntry::new(
            "text/uri-list",
            gtk::TargetFlags::OTHER_APP,
            0,
        )];
        self.window
            .drag_dest_set(gtk::DestDefaults::ALL, &targets, gdk::DragAction::COPY);

        let state = self.state.clone();
        let stack = self.main_stack.clone();
        let content_box = self.content_box.clone();
        let preview_box = self.preview_box.clone();
        let window_clone = self.window.clone();

        self.window.connect_drag_data_received(move |_win, _ctx, _x, _y, data, _info, _time| {
            let uris = data.uris();
            let mut changed = false;

            for uri in uris {
                if let Ok((path, _)) = glib::filename_from_uri(&uri) {
                    if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                        if let Some(sc) = ShortcutManager::from_desktop_file(&path) {
                            let mut st = state.borrow_mut();
                            if st.shortcuts.add(sc) {
                                changed = true;
                            }
                        }
                    } else if ImageManager::is_image_path(&path) {
                        let mut st = state.borrow_mut();
                        if st.images.add_from_path(&path).is_ok() {
                            changed = true;
                        }
                    }
                }
            }

            if changed {
                Self::rebuild_content(&content_box, &preview_box, &stack, &state, &window_clone);
            }
        });
    }

    pub fn render(&self) {
        Self::rebuild_content(
            &self.content_box,
            &self.preview_box,
            &self.main_stack,
            &self.state,
            &self.window,
        );
    }

    fn rebuild_content(
        content_box: &gtk::Box,
        preview_box: &gtk::Box,
        stack: &gtk::Stack,
        state: &Rc<RefCell<AppState>>,
        window: &gtk::Window,
    ) {
        for child in content_box.children() {
            content_box.remove(&child);
        }

        let st = state.borrow();
        let total_count = st.shortcuts.count() + st.images.count();
        let active_view = st.active_view.clone();
        drop(st);

        // Header
        let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header_box.style_context().add_class("qs-header");

        let title_label = gtk::Label::new(Some("󱂬 Quick Shelf"));
        title_label.style_context().add_class("qs-title");
        header_box.pack_start(&title_label, false, false, 0);

        let badge = gtk::Label::new(Some(&format!("{}", total_count)));
        badge.style_context().add_class("qs-badge");
        header_box.pack_start(&badge, false, false, 0);

        // View mode pills
        let view_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let btn_both = gtk::Button::with_label("All");
        let btn_apps = gtk::Button::with_label("Apps");
        let btn_media = gtk::Button::with_label("Media");

        btn_both.style_context().add_class("qs-btn");
        btn_apps.style_context().add_class("qs-btn");
        btn_media.style_context().add_class("qs-btn");

        match active_view.as_str() {
            "apps" => btn_apps.style_context().add_class("qs-tab-active"),
            "media" => btn_media.style_context().add_class("qs-tab-active"),
            _ => btn_both.style_context().add_class("qs-tab-active"),
        }

        let s1 = state.clone();
        let c1 = content_box.clone();
        let p1 = preview_box.clone();
        let st1 = stack.clone();
        let w1 = window.clone();
        btn_both.connect_clicked(move |_| {
            s1.borrow_mut().active_view = "both".to_string();
            Self::rebuild_content(&c1, &p1, &st1, &s1, &w1);
        });

        let s2 = state.clone();
        let c2 = content_box.clone();
        let p2 = preview_box.clone();
        let st2 = stack.clone();
        let w2 = window.clone();
        btn_apps.connect_clicked(move |_| {
            s2.borrow_mut().active_view = "apps".to_string();
            Self::rebuild_content(&c2, &p2, &st2, &s2, &w2);
        });

        let s3 = state.clone();
        let c3 = content_box.clone();
        let p3 = preview_box.clone();
        let st3 = stack.clone();
        let w3 = window.clone();
        btn_media.connect_clicked(move |_| {
            s3.borrow_mut().active_view = "media".to_string();
            Self::rebuild_content(&c3, &p3, &st3, &s3, &w3);
        });

        view_box.pack_start(&btn_both, false, false, 0);
        view_box.pack_start(&btn_apps, false, false, 0);
        view_box.pack_start(&btn_media, false, false, 0);
        header_box.pack_end(&view_box, false, false, 0);

        content_box.pack_start(&header_box, false, false, 0);

        // Sections
        if active_view == "both" || active_view == "apps" {
            Self::build_shortcuts_section(content_box, preview_box, stack, state, window);
        }

        if active_view == "both" {
            let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
            content_box.pack_start(&sep, false, false, 4);
        }

        if active_view == "both" || active_view == "media" {
            Self::build_media_section(content_box, preview_box, stack, state, window);
        }

        content_box.show_all();
        stack.set_visible_child_name("content");
    }

    fn build_shortcuts_section(
        content_box: &gtk::Box,
        preview_box: &gtk::Box,
        stack: &gtk::Stack,
        state: &Rc<RefCell<AppState>>,
        window: &gtk::Window,
    ) {
        let sec_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let title = gtk::Label::new(Some("Shortcuts"));
        title.style_context().add_class("qs-section-title");
        sec_header.pack_start(&title, false, false, 0);

        let add_btn = gtk::Button::with_label("+ Add");
        add_btn.style_context().add_class("qs-btn");
        let s_clone = state.clone();
        let c_clone = content_box.clone();
        let p_clone = preview_box.clone();
        let st_clone = stack.clone();
        let w_clone = window.clone();
        add_btn.connect_clicked(move |_| {
            Self::show_app_picker(&s_clone, &c_clone, &p_clone, &st_clone, &w_clone);
        });
        sec_header.pack_end(&add_btn, false, false, 0);
        content_box.pack_start(&sec_header, false, false, 0);

        let flow = gtk::FlowBox::new();
        flow.set_valign(gtk::Align::Start);
        flow.set_max_children_per_line(5);
        flow.set_min_children_per_line(3);
        flow.set_selection_mode(gtk::SelectionMode::None);
        flow.set_row_spacing(6);
        flow.set_column_spacing(6);

        let shortcuts = state.borrow().shortcuts.list().to_vec();
        for item in shortcuts {
            let card = gtk::Button::new();
            card.style_context().add_class("qs-app-card");

            let card_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
            card_box.set_halign(gtk::Align::Center);

            let icon_img = if gtk::IconTheme::default()
                .map(|t| t.has_icon(&item.icon))
                .unwrap_or(false)
            {
                gtk::Image::from_icon_name(Some(&item.icon), gtk::IconSize::Dnd)
            } else {
                gtk::Image::from_icon_name(Some("application-x-executable"), gtk::IconSize::Dnd)
            };
            card_box.pack_start(&icon_img, false, false, 0);

            let label = gtk::Label::new(Some(&item.name));
            label.style_context().add_class("qs-app-label");
            label.set_ellipsize(pango::EllipsizeMode::End);
            label.set_max_width_chars(8);
            card_box.pack_start(&label, false, false, 0);

            card.add(&card_box);
            card.set_tooltip_text(Some(&format!("{}\nExec: {}", item.name, item.exec)));

            let exec_cmd = item.exec.clone();
            card.connect_clicked(move |_| {
                let _ = ShortcutManager::launch_exec(&exec_cmd);
            });

            // Right-click context menu
            let s_rm = state.clone();
            let c_rm = content_box.clone();
            let p_rm = preview_box.clone();
            let st_rm = stack.clone();
            let w_rm = window.clone();
            let item_id = item.id.clone();
            let item_exec = item.exec.clone();

            card.connect_button_press_event(move |_btn, event| {
                if event.button() == 3 {
                    let menu = gtk::Menu::new();
                    let mi_launch = gtk::MenuItem::with_label("Launch");
                    let e_launch = item_exec.clone();
                    mi_launch.connect_activate(move |_| {
                        let _ = ShortcutManager::launch_exec(&e_launch);
                    });

                    let mi_copy = gtk::MenuItem::with_label("Copy Command");
                    let e_copy = item_exec.clone();
                    mi_copy.connect_activate(move |_| {
                        let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                        clipboard.set_text(&e_copy);
                    });

                    let mi_del = gtk::MenuItem::with_label("Remove Shortcut");
                    let id_del = item_id.clone();
                    let s_del = s_rm.clone();
                    let c_del = c_rm.clone();
                    let p_del = p_rm.clone();
                    let st_del = st_rm.clone();
                    let w_del = w_rm.clone();
                    mi_del.connect_activate(move |_| {
                        s_del.borrow_mut().shortcuts.remove(&id_del);
                        Self::rebuild_content(&c_del, &p_del, &st_del, &s_del, &w_del);
                    });

                    menu.append(&mi_launch);
                    menu.append(&mi_copy);
                    menu.append(&gtk::SeparatorMenuItem::new());
                    menu.append(&mi_del);
                    menu.show_all();
                    menu.popup_at_pointer(Some(event));
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });

            flow.add(&card);
        }

        content_box.pack_start(&flow, false, false, 0);
    }

    fn build_media_section(
        content_box: &gtk::Box,
        preview_box: &gtk::Box,
        stack: &gtk::Stack,
        state: &Rc<RefCell<AppState>>,
        window: &gtk::Window,
    ) {
        let sec_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let title = gtk::Label::new(Some("Image Shelf"));
        title.style_context().add_class("qs-section-title");
        sec_header.pack_start(&title, false, false, 0);

        let clear_btn = gtk::Button::with_label("Clear");
        clear_btn.style_context().add_class("qs-btn");
        let s_clr = state.clone();
        let c_clr = content_box.clone();
        let p_clr = preview_box.clone();
        let st_clr = stack.clone();
        let w_clr = window.clone();
        clear_btn.connect_clicked(move |_| {
            s_clr.borrow_mut().images.clear_all();
            Self::rebuild_content(&c_clr, &p_clr, &st_clr, &s_clr, &w_clr);
        });
        sec_header.pack_end(&clear_btn, false, false, 0);

        let pick_btn = gtk::Button::with_label("+ Pick");
        pick_btn.style_context().add_class("qs-btn");
        let s_pick = state.clone();
        let c_pick = content_box.clone();
        let p_pick = preview_box.clone();
        let st_pick = stack.clone();
        let w_pick = window.clone();
        pick_btn.connect_clicked(move |_| {
            let dialog = gtk::FileChooserDialog::with_buttons(
                Some("Select Image"),
                Some(&w_pick),
                gtk::FileChooserAction::Open,
                &[
                    ("Cancel", gtk::ResponseType::Cancel),
                    ("Open", gtk::ResponseType::Accept),
                ],
            );
            let filter = gtk::FileFilter::new();
            filter.set_name(Some("Images"));
            filter.add_pixbuf_formats();
            dialog.add_filter(filter);

            if dialog.run() == gtk::ResponseType::Accept {
                if let Some(filename) = dialog.filename() {
                    let mut st = s_pick.borrow_mut();
                    let _ = st.images.add_from_path(&filename);
                    drop(st);
                    Self::rebuild_content(&c_pick, &p_pick, &st_pick, &s_pick, &w_pick);
                }
            }
            dialog.close();
        });
        sec_header.pack_end(&pick_btn, false, false, 0);

        content_box.pack_start(&sec_header, false, false, 0);

        let images = state.borrow().images.list().to_vec();
        if images.is_empty() {
            let dropzone = gtk::Box::new(gtk::Orientation::Vertical, 4);
            dropzone.style_context().add_class("qs-dropzone");
            dropzone.set_halign(gtk::Align::Fill);

            let icon = gtk::Image::from_icon_name(Some("insert-image"), gtk::IconSize::Dialog);
            dropzone.pack_start(&icon, false, false, 0);

            let lbl = gtk::Label::new(Some("Drop images here from file manager or browser"));
            lbl.style_context().add_class("qs-app-label");
            lbl.set_line_wrap(true);
            dropzone.pack_start(&lbl, false, false, 0);

            content_box.pack_start(&dropzone, false, false, 0);
        } else {
            let scrolled = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
            scrolled.set_min_content_height(100);
            scrolled.set_max_content_height(240);
            scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

            let flow = gtk::FlowBox::new();
            flow.set_valign(gtk::Align::Start);
            flow.set_max_children_per_line(4);
            flow.set_min_children_per_line(3);
            flow.set_selection_mode(gtk::SelectionMode::None);
            flow.set_row_spacing(8);
            flow.set_column_spacing(8);

            for item in images {
                let card = gtk::Button::new();
                card.style_context().add_class("qs-image-card");

                let thumb_path = item.path.clone();
                let image_widget = if let Ok(pb) = Pixbuf::from_file_at_scale(&thumb_path, 76, 76, true) {
                    gtk::Image::from_pixbuf(Some(&pb))
                } else {
                    gtk::Image::from_icon_name(Some("image-missing"), gtk::IconSize::Dialog)
                };

                card.add(&image_widget);
                card.set_tooltip_text(Some(&format!(
                    "{}\n{}x{} ({} KB)",
                    item.name,
                    item.width,
                    item.height,
                    item.size_bytes / 1024
                )));

                // Left-click -> Instant Preview Lightbox
                let it_prev = item.clone();
                let s_prev = state.clone();
                let c_prev = content_box.clone();
                let p_prev = preview_box.clone();
                let st_prev = stack.clone();
                let w_prev = window.clone();
                card.connect_clicked(move |_| {
                    Self::show_preview(&it_prev, &s_prev, &c_prev, &p_prev, &st_prev, &w_prev);
                });

                // Setup Drag OUT (Drag Source)
                let drag_path = item.path.clone();
                let targets = [gtk::TargetEntry::new(
                    "text/uri-list",
                    gtk::TargetFlags::OTHER_APP,
                    0,
                )];
                card.drag_source_set(gdk::ModifierType::BUTTON1_MASK, &targets, gdk::DragAction::COPY);

                let d_path = drag_path.clone();
                card.connect_drag_data_get(move |_widget, _ctx, data, _info, _time| {
                    if let Ok(uri) = glib::filename_to_uri(&d_path, None) {
                        data.set_uris(&[&uri]);
                    }
                });

                // Context menu
                let it_ctx = item.clone();
                let s_ctx = state.clone();
                let c_ctx = content_box.clone();
                let p_ctx = preview_box.clone();
                let st_ctx = stack.clone();
                let w_ctx = window.clone();

                card.connect_button_press_event(move |_btn, event| {
                    if event.button() == 3 {
                        let menu = gtk::Menu::new();
                        let mi_preview = gtk::MenuItem::with_label("Instant Preview");
                        let it_p = it_ctx.clone();
                        let s_p = s_ctx.clone();
                        let c_p = c_ctx.clone();
                        let p_p = p_ctx.clone();
                        let st_p = st_ctx.clone();
                        let w_p = w_ctx.clone();
                        mi_preview.connect_activate(move |_| {
                            Self::show_preview(&it_p, &s_p, &c_p, &p_p, &st_p, &w_p);
                        });

                        let mi_copy_img = gtk::MenuItem::with_label("Copy Image Data");
                        let p_copy = PathBuf::from(&it_ctx.path);
                        mi_copy_img.connect_activate(move |_| {
                            ImageManager::copy_image_to_clipboard(&p_copy);
                        });

                        let mi_copy_path = gtk::MenuItem::with_label("Copy File Path");
                        let p_path = PathBuf::from(&it_ctx.path);
                        mi_copy_path.connect_activate(move |_| {
                            ImageManager::copy_path_to_clipboard(&p_path);
                        });

                        let mi_open = gtk::MenuItem::with_label("Open in Default Viewer");
                        let p_open = PathBuf::from(&it_ctx.path);
                        mi_open.connect_activate(move |_| {
                            ImageManager::open_image(&p_open);
                        });

                        let mi_del = gtk::MenuItem::with_label("Remove from Shelf");
                        let id_del = it_ctx.id.clone();
                        let s_del = s_ctx.clone();
                        let c_del = c_ctx.clone();
                        let p_del = p_ctx.clone();
                        let st_del = st_ctx.clone();
                        let w_del = w_ctx.clone();
                        mi_del.connect_activate(move |_| {
                            s_del.borrow_mut().images.remove(&id_del);
                            Self::rebuild_content(&c_del, &p_del, &st_del, &s_del, &w_del);
                        });

                        menu.append(&mi_preview);
                        menu.append(&mi_copy_img);
                        menu.append(&mi_copy_path);
                        menu.append(&mi_open);
                        menu.append(&gtk::SeparatorMenuItem::new());
                        menu.append(&mi_del);
                        menu.show_all();
                        menu.popup_at_pointer(Some(event));
                        return glib::Propagation::Stop;
                    }
                    glib::Propagation::Proceed
                });

                flow.add(&card);
            }

            scrolled.add(&flow);
            content_box.pack_start(&scrolled, true, true, 0);
        }
    }

    fn show_preview(
        item: &ImageItem,
        state: &Rc<RefCell<AppState>>,
        content_box: &gtk::Box,
        preview_box: &gtk::Box,
        stack: &gtk::Stack,
        window: &gtk::Window,
    ) {
        for child in preview_box.children() {
            preview_box.remove(&child);
        }

        let top_bar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let back_btn = gtk::Button::with_label("← Back");
        back_btn.style_context().add_class("qs-btn");
        let st_back = stack.clone();
        back_btn.connect_clicked(move |_| {
            st_back.set_visible_child_name("content");
        });
        top_bar.pack_start(&back_btn, false, false, 0);

        let title = gtk::Label::new(Some(&item.name));
        title.style_context().add_class("qs-title");
        title.set_ellipsize(pango::EllipsizeMode::End);
        top_bar.pack_start(&title, true, true, 0);

        preview_box.pack_start(&top_bar, false, false, 0);

        // Scaled preview image
        let p = Path::new(&item.path);
        if let Ok(pb) = Pixbuf::from_file_at_scale(p, 340, 240, true) {
            let img = gtk::Image::from_pixbuf(Some(&pb));
            preview_box.pack_start(&img, false, false, 6);
        }

        // Metadata label
        let meta_text = format!(
            "Resolution: {} × {} px   |   Size: {:.1} KB",
            item.width,
            item.height,
            (item.size_bytes as f64) / 1024.0
        );
        let meta_label = gtk::Label::new(Some(&meta_text));
        meta_label.style_context().add_class("qs-app-label");
        preview_box.pack_start(&meta_label, false, false, 0);

        // Action buttons
        let actions_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        actions_box.set_halign(gtk::Align::Center);

        let btn_copy_img = gtk::Button::with_label("Copy Image");
        btn_copy_img.style_context().add_class("qs-btn");
        let p_img = p.to_path_buf();
        btn_copy_img.connect_clicked(move |_| {
            ImageManager::copy_image_to_clipboard(&p_img);
        });

        let btn_copy_path = gtk::Button::with_label("Copy Path");
        btn_copy_path.style_context().add_class("qs-btn");
        let p_path = p.to_path_buf();
        btn_copy_path.connect_clicked(move |_| {
            ImageManager::copy_path_to_clipboard(&p_path);
        });

        let btn_open = gtk::Button::with_label("Open Viewer");
        btn_open.style_context().add_class("qs-btn");
        let p_open = p.to_path_buf();
        btn_open.connect_clicked(move |_| {
            ImageManager::open_image(&p_open);
        });

        let btn_del = gtk::Button::with_label("Delete");
        btn_del.style_context().add_class("qs-btn");
        let id_del = item.id.clone();
        let s_del = state.clone();
        let c_del = content_box.clone();
        let pr_del = preview_box.clone();
        let st_del = stack.clone();
        let w_del = window.clone();
        btn_del.connect_clicked(move |_| {
            s_del.borrow_mut().images.remove(&id_del);
            Self::rebuild_content(&c_del, &pr_del, &st_del, &s_del, &w_del);
        });

        actions_box.pack_start(&btn_copy_img, false, false, 0);
        actions_box.pack_start(&btn_copy_path, false, false, 0);
        actions_box.pack_start(&btn_open, false, false, 0);
        actions_box.pack_start(&btn_del, false, false, 0);

        preview_box.pack_start(&actions_box, false, false, 6);

        preview_box.show_all();
        stack.set_visible_child_name("preview");
    }

    fn show_app_picker(
        state: &Rc<RefCell<AppState>>,
        content_box: &gtk::Box,
        preview_box: &gtk::Box,
        stack: &gtk::Stack,
        window: &gtk::Window,
    ) {
        let dialog = gtk::Dialog::with_buttons(
            Some("Add App Shortcut"),
            Some(window),
            gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
            &[("Close", gtk::ResponseType::Close)],
        );
        dialog.set_default_size(360, 420);

        let content_area = dialog.content_area();
        content_area.set_spacing(8);

        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search applications..."));
        content_area.pack_start(&search_entry, false, false, 4);

        let scrolled = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
        let list_box = gtk::ListBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::Single);

        let apps = ShortcutManager::list_system_apps();
        for app in apps {
            let row = gtk::ListBoxRow::new();
            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            row_box.set_border_width(6);

            let icon_img = if gtk::IconTheme::default()
                .map(|t| t.has_icon(&app.icon))
                .unwrap_or(false)
            {
                gtk::Image::from_icon_name(Some(&app.icon), gtk::IconSize::Button)
            } else {
                gtk::Image::from_icon_name(Some("application-x-executable"), gtk::IconSize::Button)
            };
            row_box.pack_start(&icon_img, false, false, 0);

            let name_lbl = gtk::Label::new(Some(&app.name));
            name_lbl.set_halign(gtk::Align::Start);
            row_box.pack_start(&name_lbl, true, true, 0);

            row.add(&row_box);
            row.set_tooltip_text(Some(&app.exec));

            let s_app = state.clone();
            let c_app = content_box.clone();
            let p_app = preview_box.clone();
            let st_app = stack.clone();
            let w_app = window.clone();
            let d_app = dialog.clone();
            let sc_add = app.clone();

            row.connect_activate(move |_| {
                s_app.borrow_mut().shortcuts.add(sc_add.clone());
                Self::rebuild_content(&c_app, &p_app, &st_app, &s_app, &w_app);
                d_app.response(gtk::ResponseType::Close);
            });

            list_box.add(&row);
        }

        // Search filtering
        let lb_clone = list_box.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_lowercase();
            lb_clone.set_filter_func(Some(Box::new(move |row| {
                if query.is_empty() {
                    return true;
                }
                if let Some(tooltip) = row.tooltip_text() {
                    if tooltip.to_lowercase().contains(&query) {
                        return true;
                    }
                }
                if let Some(w) = row.child() {
                    if let Ok(b) = w.downcast::<gtk::Box>() {
                        for c in b.children() {
                            if let Ok(l) = c.downcast::<gtk::Label>() {
                                if l.text().to_lowercase().contains(&query) {
                                    return true;
                                }
                            }
                        }
                    }
                }
                false
            })));
        });

        scrolled.add(&list_box);
        content_area.pack_start(&scrolled, true, true, 0);

        dialog.show_all();
        dialog.run();
        dialog.close();
    }
}
