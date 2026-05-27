use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, EventControllerKey, Image, Label, ListBox,
    ListBoxRow, Orientation, ScrolledWindow, SearchEntry, glib,
};
use gtk4_layer_shell::{Layer, LayerShell};
use std::fs;

use gtk4::EventControllerFocus;
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "dev.codingkittend.twinkl";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let search_bar = SearchEntry::builder().build();

    let listbox = ListBox::builder().build();

    let apps = parse_apps();

    for entry in &apps {
        let hbox = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        if let Some(icon_name) = &entry.icon {
            let icon = Image::from_icon_name(icon_name);
            icon.set_pixel_size(32);
            hbox.append(&icon);
        }

        let label = Label::builder().label(&entry.name).build();
        hbox.append(&label);

        let row = ListBoxRow::builder().child(&hbox).build();
        listbox.append(&row);
    }

    let search_bar_clone = search_bar.clone();
    listbox.set_filter_func(move |row| {
        let query = search_bar_clone.text().to_lowercase();
        if query.is_empty() {
            return true; // show everything when search is empty
        }
        // get the label text from inside the row
        let hbox = row.child().unwrap().downcast::<GtkBox>().unwrap();
        let label = hbox.last_child().unwrap().downcast::<Label>().unwrap();
        let row_text = label.text().to_lowercase();
        row_text.contains(&query)
    });

    let listbox_clone = listbox.clone();
    search_bar.connect_changed(move |_| {
        listbox_clone.invalidate_filter();
    });

    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    let scrollable = ScrolledWindow::builder()
        .kinetic_scrolling(true)
        .min_content_width(300)
        .min_content_height(300)
        .child(&listbox)
        .build();

    vbox.append(&search_bar);
    vbox.append(&scrollable);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("twinkl")
        .child(&vbox)
        .build();

    let key_controller = EventControllerKey::new();
    let window_clone = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            window_clone.close();
        }
        glib::Propagation::Proceed
    });

    let pending_key: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let pending_key_clone = pending_key.clone();
    let search_bar_clone = search_bar.clone();

    let list_key_controller = EventControllerKey::new();
    list_key_controller.connect_key_pressed(move |_, key, _, _| match key {
        gtk4::gdk::Key::Return
        | gtk4::gdk::Key::Escape
        | gtk4::gdk::Key::Up
        | gtk4::gdk::Key::Down
        | gtk4::gdk::Key::Left
        | gtk4::gdk::Key::Right => glib::Propagation::Proceed,
        _ => {
            if let Some(ch) = key.to_unicode() {
                *pending_key_clone.borrow_mut() = Some(ch.to_string());
            }
            search_bar_clone.grab_focus();
            glib::Propagation::Stop
        }
    });
    listbox.add_controller(list_key_controller);

    let pending_key_clone2 = pending_key.clone();
    let focus_controller = EventControllerFocus::new();
    focus_controller.connect_enter(move |controller| {
        if let Some(ch) = pending_key_clone2.borrow_mut().take() {
            let entry = controller
                .widget()
                .unwrap()
                .downcast::<SearchEntry>()
                .unwrap();
            entry.insert_text(&ch, &mut -1);
            entry.set_position(-1);
        }
    });
    search_bar.add_controller(focus_controller);

    let window_clone2 = window.clone();
    let search_key_controller = EventControllerKey::new();
    search_key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            window_clone2.close();
        }
        glib::Propagation::Proceed
    });
    search_bar.add_controller(search_key_controller);

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    window.add_controller(key_controller);

    window.present();
}

struct AppEntry {
    name: String,
    exec: String,
    icon: Option<String>,
}

fn parse_apps() -> Vec<AppEntry> {
    let mut apps = Vec::new();

    let dirs = fs::read_dir("/usr/share/applications").unwrap();

    for entry in dirs {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
            continue; // skip non-.desktop files
        }

        let contents = fs::read_to_string(&path).unwrap();
        let mut name = None;
        let mut exec = None;
        let mut icon = None;

        for line in contents.lines() {
            if line.starts_with('#') || line.starts_with('[') {
                continue; // skip comments and section headers like [Desktop Entry]
            }
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Name" => name = Some(value.trim().to_string()),
                    "Exec" => {
                        exec = Some(
                            value
                                .trim()
                                .replace("%u", "")
                                .replace("%f", "")
                                .replace("%U", "")
                                .replace("%F", "")
                                .trim()
                                .to_string(),
                        )
                    }
                    "Icon" => icon = Some(value.trim().to_string()),
                    _ => {} // ignore everything else
                }
            }
        }

        if let (Some(name), Some(exec)) = (name, exec) {
            apps.push(AppEntry { name, exec, icon });
        }
    }

    apps
}
