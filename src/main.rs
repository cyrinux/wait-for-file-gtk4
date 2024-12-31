use clap::Parser;
use gdk4::Key;
use glib::MainContext;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Image, Label, Orientation, ProgressBar,
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

fn create_main_box(
    icon_spec: Option<String>,
    presence_file: &str,
    extra_label: &str,
) -> (GtkBox, ProgressBar, Button, Button) {
    let main_box = GtkBox::new(Orientation::Horizontal, 10);
    main_box.set_margin_end(20);

    if let Some(icon_spec) = icon_spec {
        let image = if std::path::Path::new(&icon_spec).exists() {
            Image::from_file(icon_spec)
        } else {
            Image::from_icon_name(&icon_spec)
        };
        image.set_icon_size(gtk4::IconSize::Large);
        image.set_margin_start(20);
        main_box.append(&image);
    }

    let vbox = GtkBox::new(Orientation::Vertical, 10);
    let label = Label::new(Some(&format!("Waiting for file: {}", presence_file)));
    label.set_margin_bottom(10);
    label.set_margin_top(10);
    vbox.append(&label);

    let progress_bar = ProgressBar::new();
    progress_bar.set_show_text(false);
    progress_bar.set_margin_bottom(10);
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
        let state = Arc::new(AppState {
            presence_file: args.presence_file.clone(),
            main_command: args.command.clone(),
            extra_command: extra_command.clone(),
            is_running: Arc::new(AtomicBool::new(true)),
        });

        let (tx_file_found, rx_file_found) = MainContext::channel(glib::PRIORITY_DEFAULT);
        setup_file_watcher(Arc::clone(&state), tx_file_found);

        let (main_box, progress_bar, button_extra, button_cancel) =
            create_main_box(args.icon.clone(), &state.presence_file, &extra_label);

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
