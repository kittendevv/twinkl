use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, CssProvider, EventControllerKey, Image, Label,
    ListBox, ListBoxRow, Orientation, ScrolledWindow, SearchEntry, glib,
};
use gtk4_layer_shell::{Layer, LayerShell};
use std::fs;

use gtk4::EventControllerFocus;
use std::cell::RefCell;
use std::rc::Rc;

use freedesktop_file_parser::{EntryType, parse};

const APP_ID: &str = "dev.codingkittend.twinkl";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn load_css() {
    let provider = CssProvider::new();

    let css_path = dirs::config_dir().unwrap().join("twinkl").join("style.css");

    if css_path.exists() {
        provider.load_from_path(css_path);
    } else {
        // fallback hardcoded style
        provider.load_from_string(
            "
            entry,
            entry text,
            entry undershoot.left,
            entry undershoot.right {
                background-color: #011936;
                color: #D9F0FF;
                border: none;1
                box-shadow: none;
                outline: none;
            }

            entry:focus,
            entry:focus-within {
               	box-shadow: none;
               	outline: none;
            }

            image {
                color: #D9F0FF;
            }

            window {
                background: transparent;
            }

            .launcher {
               	background-color: #011936;
               	border-radius: 12px;
               	padding: 8px;
            }

            list {
           	    background: transparent;
            }

            row {
                border-radius: 8px;
                padding: 4px;
            }

            row:selected {
                background-color: #A3D5FF;
                outline: none;
            }

            row:hover {
                color: #A3D5FF;
                outline: none;
            }

            row:selected label {
                color: #011936;
            }

            label {
                color: #D9F0FF;
            }
        ",
        );
    }

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application) {
    load_css();

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
        .min_content_width(500)
        .min_content_height(300)
        .child(&listbox)
        .build();

    vbox.append(&search_bar);
    vbox.append(&scrollable);

    let window = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
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

    let apps_clone = apps.clone();
    let window_clone = window.clone();
    listbox.connect_row_activated(move |_, row| {
        let index = row.index() as usize;
        if let Some(app) = apps_clone.get(index) {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(&app.exec)
                .spawn();
            window_clone.close();
        }
    });

    let window_clone2 = window.clone();
    let search_key_controller = EventControllerKey::new();
    search_key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            window_clone2.close();
        }
        glib::Propagation::Proceed
    });
    search_bar.add_controller(search_key_controller);

    vbox.add_css_class("launcher");

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    window.add_controller(key_controller);

    window.present();
}

#[derive(Clone)]
struct AppEntry {
    name: String,
    exec: String,
    icon: Option<String>,
}

fn parse_apps() -> Vec<AppEntry> {
    let mut apps = Vec::new();

    let mut dirs = vec![std::path::PathBuf::from("/usr/share/applications")];
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(std::path::PathBuf::from(home).join(".local/share/applications"));
    }

    for dir in dirs {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue, // skip if dir doesn't exist
        };

        for entry in entries {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }

            let contents = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let desktop_file = match parse(&contents) {
                Ok(d) => d,
                Err(_) => continue,
            };

            // skip non-application entries (e.g. links, directories)
            let exec = if let EntryType::Application(app) = &desktop_file.entry.entry_type {
                app.exec.clone()
            } else {
                continue;
            };

            let name = desktop_file.entry.name.default.clone();
            if desktop_file.entry.no_display.unwrap_or(false)
                || desktop_file.entry.hidden.unwrap_or(false)
            {
                continue;
            }
            let icon = desktop_file.entry.icon.as_ref().map(|i| i.content.clone());

            apps.push(AppEntry {
                name,
                exec: exec.unwrap_or_default(),
                icon,
            });
        }
    }

    apps
}
