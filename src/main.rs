use clap::Parser;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, ProgressBar,
};
use glib::MainContext;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

/// Command-line arguments for our wait-for-file app.
#[derive(Parser, Debug)]
#[command(
    name = "wait_for_file",
    about = "A GTK4 app that waits for a file, then runs a command.\nAlso supports a customizable 'Unlock' button."
)]
struct Args {
    /// File path to wait for (once it appears, we run --command)
    #[arg(short, long)]
    pub presence_file: String,

    /// The main command to run once the file is found
    #[arg(short, long)]
    pub command: String,

    /// Customizable unlock button in the format "Label:Command"
    /// (e.g., "Unlock:open-vault"). Defaults to "Unlock:open-vault".
    #[arg(short, long, default_value = "Unlock:open-vault")]
    pub unlock_command: String,
}

fn parse_unlock_command(unlock_command: &str) -> (String, String) {
    let parts: Vec<&str> = unlock_command.splitn(2, ':').collect();
    if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        ("Unlock".to_string(), unlock_command.to_string())
    }
}

fn main() {
    // Parse CLI arguments with Clap
    let args = Args::parse();

    // Parse the unlock command into label and command
    let (unlock_label, unlock_command) = parse_unlock_command(&args.unlock_command);

    // Create the GTK4 application
    let app = Application::builder()
        .application_id("name.levis.waitforfile.gtk4")
        .build();

    // Move arguments into the connect_activate closure
    app.connect_activate(move |app| {
        let presence_file = args.presence_file.clone();
        let main_command = args.command.clone();
        let unlock_label = unlock_label.clone();
        let unlock_command = unlock_command.clone();

        // -------------------------------------------------------------------
        // A) Create a glib channel + background thread in the same scope
        // -------------------------------------------------------------------
        let (tx_file_found, rx_file_found) = MainContext::channel::<()>(glib::PRIORITY_DEFAULT);
        let is_running = Arc::new(AtomicBool::new(true));

        {
            let presence_file_clone = presence_file.clone();
            let main_command_clone = main_command.clone();
            let is_running_clone = Arc::clone(&is_running);

            thread::spawn(move || {
                while is_running_clone.load(Ordering::SeqCst) {
                    if std::path::Path::new(&presence_file_clone).exists() {
                        let _ = Command::new("sh")
                            .arg("-c")
                            .arg(&main_command_clone)
                            .spawn();

                        let _ = tx_file_found.send(());
                        break;
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            });
        }

        // -------------------------------------------------------------------
        // B) Build the GUI
        // -------------------------------------------------------------------
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Waiting for File")
            .default_width(400)
            .default_height(250)
            .build();

        let vbox = GtkBox::new(Orientation::Vertical, 10);
        vbox.set_margin_top(20);
        vbox.set_margin_bottom(20);
        vbox.set_margin_start(20);
        vbox.set_margin_end(20);

        let label = Label::new(Some(&format!(
            "Waiting for file: {}",
            presence_file
        )));
        label.set_margin_bottom(10);
        vbox.append(&label);

        let progress_bar = ProgressBar::new();
        progress_bar.set_show_text(false);
        progress_bar.set_margin_bottom(10);
        vbox.append(&progress_bar);

        let hbox = GtkBox::new(Orientation::Horizontal, 10);
        hbox.set_halign(gtk4::Align::Center);

        let button_unlock = Button::with_label(&unlock_label);
        hbox.append(&button_unlock);

        let button_cancel = Button::with_label("Cancel");
        hbox.append(&button_cancel);

        vbox.append(&hbox);
        window.set_child(Some(&vbox));
        window.show();

        // B1) If the custom button is clicked => run the corresponding command
        {
            let unlock_command_clone = unlock_command.clone();
            button_unlock.connect_clicked(move |_btn| {
                let cmd = unlock_command_clone.clone();
                thread::spawn(move || {
                    let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
                });
            });
        }

        // B2) If "Cancel" is clicked => stop background thread + quit
        {
            let app_clone = app.clone();
            let is_running_clone = Arc::clone(&is_running);
            button_cancel.connect_clicked(move |_btn| {
                is_running_clone.store(false, Ordering::SeqCst);
                app_clone.quit();
            });
        }

        // B3) Pulse the progress bar in the main thread
        glib::timeout_add_local(Duration::from_millis(300), move || {
            progress_bar.pulse();
            glib::Continue(true)
        });

        // B4) Attach the channel receiver => once we get a "file found" signal => quit
        {
            let app_clone = app.clone();
            rx_file_found.attach(None, move |_| {
                app_clone.quit();
                glib::Continue(false)
            });
        }
    });

    // Run the GTK application
    app.run_with_args::<glib::GString>(&[]);
}
