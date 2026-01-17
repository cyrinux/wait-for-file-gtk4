use clap::Parser;
use gdk4::Key;
use gio::prelude::{AppInfoExt, FileExt};
use gio::{AppInfo, FileIcon, ThemedIcon};
use glib::{Cast, MainContext};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, IconTheme, Image, Label, Orientation,
    ProgressBar,
};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "wait_for_file",
    about = "A GTK4 app that waits for a file, then runs a command.\nAlso supports a extra customizable button."
)]
struct Args {
    #[arg(short, long)]
    pub presence_file: String,

    #[arg(short, long)]
    pub command: String,

    #[arg(short, long, default_value = "Unlock:open-vault 120s")]
    pub extra_command: String,

    #[arg(short, long)]
    pub icon: Option<String>,

    #[arg(
        long,
        help = "Disable automatic triggering of the unlock command on startup"
    )]
    pub no_auto_unlock: bool,
}

struct AppState {
    presence_file: String,
    main_command: String,
    extra_command: String,
    is_running: Arc<AtomicBool>,
}

struct GuiComponents {
    app: Application,
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "
        .main-container {
            padding: 12px;
        }
        window {
            border: 1px solid alpha(@accent_bg_color, 0.6);
            border-radius: 6px;
        }
        ",
    );

    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().expect("Could not get default display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn create_main_box(
    icon_spec: Option<String>,
    presence_file: &str,
    extra_label: &str,
    display: &gdk4::Display,
) -> (GtkBox, ProgressBar, Button, Button) {
    let main_box = GtkBox::new(Orientation::Horizontal, 12);
    main_box.add_css_class("main-container");
    main_box.set_valign(gtk4::Align::Center);
    main_box.set_hexpand(true);
    main_box.set_vexpand(true);

    if let Some(icon_spec) = icon_spec {
        if std::path::Path::new(&icon_spec).exists() {
            let img = Image::from_file(&icon_spec);
            img.set_pixel_size(48);
            img.set_valign(gtk4::Align::Center);
            main_box.append(&img);
        } else {
            let icon_theme = IconTheme::for_display(display);
            if icon_theme.has_icon(&icon_spec) {
                let img = Image::from_icon_name(&icon_spec);
                img.set_pixel_size(48);
                img.set_valign(gtk4::Align::Center);
                main_box.append(&img);
            }
        }
    }

    let vbox = GtkBox::new(Orientation::Vertical, 8);
    vbox.set_valign(gtk4::Align::Center);
    let label = Label::new(Some(&format!("Waiting for file: {}", presence_file)));
    vbox.append(&label);

    let progress_bar = ProgressBar::new();
    progress_bar.set_show_text(false);
    vbox.append(&progress_bar);

    let hbox = GtkBox::new(Orientation::Horizontal, 10);
    hbox.set_halign(gtk4::Align::Center);

    let button_extra = Button::with_label(extra_label);
    let button_cancel = Button::with_label("Cancel");
    hbox.append(&button_extra);
    hbox.append(&button_cancel);

    vbox.append(&hbox);
    main_box.append(&vbox);

    (main_box, progress_bar, button_extra, button_cancel)
}

fn find_icon_for_command(command: &str) -> Option<String> {
    let binary_name = command.split_whitespace().next()?;

    let apps = AppInfo::all();
    for app in apps {
        let exe_path = app.executable();
        let exe_name = exe_path.file_name().and_then(|n| n.to_str());
        let match_found = exe_name.map(|n| n == binary_name).unwrap_or(false)
            || exe_path.to_str().map(|s| s == binary_name).unwrap_or(false);

        if match_found {
            if let Some(icon) = app.icon() {
                if let Some(themed) = icon.downcast_ref::<ThemedIcon>() {
                    let names = themed.names();
                    if let Some(name) = names.first() {
                        return Some(name.to_string());
                    }
                }
                if let Some(file_icon) = icon.downcast_ref::<FileIcon>() {
                    let file = file_icon.file();
                    if let Some(path) = file.path() {
                        return path.to_str().map(String::from);
                    }
                }
            }
        }
    }

    None
}

fn setup_file_watcher(state: Arc<AppState>, tx_file_found: glib::Sender<()>) {
    thread::spawn(move || {
        while state.is_running.load(Ordering::SeqCst) {
            if std::path::Path::new(&state.presence_file).exists() {
                let _ = Command::new("sh")
                    .arg("-c")
                    .arg(&state.main_command)
                    .spawn();

                let _ = tx_file_found.send(());
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}

fn setup_window_controls(
    window: &ApplicationWindow,
    state: Arc<AppState>,
    components: Arc<GuiComponents>,
    button_extra: Button,
    button_cancel: Button,
    no_auto_unlock: bool,
) {
    let key_controller = gtk4::EventControllerKey::new();
    let state_clone = Arc::clone(&state);
    let components_clone = Arc::clone(&components);

    key_controller.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == Key::Escape {
            state_clone.is_running.store(false, Ordering::SeqCst);
            components_clone.app.quit();
            gtk4::Inhibit(true)
        } else {
            gtk4::Inhibit(false)
        }
    });
    window.add_controller(key_controller);

    let extra_command = state.extra_command.clone();
    button_extra.connect_clicked(move |_| {
        let cmd = extra_command.clone();
        thread::spawn(move || {
            let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
        });
    });

    // Auto-trigger the unlock button unless disabled
    if !no_auto_unlock {
        let cmd = state.extra_command.clone();
        thread::spawn(move || {
            let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
        });
    }

    let state_clone = Arc::clone(&state);
    let components_clone = Arc::clone(&components);
    button_cancel.connect_clicked(move |_| {
        state_clone.is_running.store(false, Ordering::SeqCst);
        components_clone.app.quit();
    });
}

fn parse_extra_command(extra_command: &str) -> (String, String) {
    let parts: Vec<&str> = extra_command.splitn(2, ':').collect();
    if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        ("Unlock".to_string(), extra_command.to_string())
    }
}

fn main() {
    let args = Args::parse();
    let (extra_label, extra_command) = parse_extra_command(&args.extra_command);

    let app = Application::builder()
        .application_id("name.levis.waitforfile.gtk4")
        .build();

    app.connect_activate(move |app| {
        load_css();

        let state = Arc::new(AppState {
            presence_file: args.presence_file.clone(),
            main_command: args.command.clone(),
            extra_command: extra_command.clone(),
            is_running: Arc::new(AtomicBool::new(true)),
        });

        let (tx_file_found, rx_file_found) = MainContext::channel(glib::PRIORITY_DEFAULT);
        setup_file_watcher(Arc::clone(&state), tx_file_found);

        let icon_spec = args
            .icon
            .clone()
            .or_else(|| find_icon_for_command(&args.command));

        let display = gdk4::Display::default().expect("Could not get default display");
        let (main_box, progress_bar, button_extra, button_cancel) =
            create_main_box(icon_spec, &state.presence_file, &extra_label, &display);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Waiting for File")
            .default_width(240)
            .default_height(120)
            .child(&main_box)
            .build();

        let components = Arc::new(GuiComponents { app: app.clone() });

        setup_window_controls(
            &window,
            Arc::clone(&state),
            Arc::clone(&components),
            button_extra,
            button_cancel,
            args.no_auto_unlock,
        );

        glib::timeout_add_local(Duration::from_millis(300), move || {
            progress_bar.pulse();
            glib::Continue(true)
        });

        let app_clone = app.clone();
        rx_file_found.attach(None, move |_| {
            app_clone.quit();
            glib::Continue(false)
        });

        window.show();
    });

    app.run_with_args::<glib::GString>(&[]);
}
