pub mod ui {
    slint::include_modules!();
}

mod app;
mod controllers {
    pub mod monitor;
}
mod manager;
mod tray;

use app::Application;

fn main() {
    let app = Application::new();
    app.run();
}
