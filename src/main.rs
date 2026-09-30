pub mod ai;
mod app;
pub mod board;
mod render;

#[macroquad::main(window_conf)]
async fn main() {
    app::run().await;
}

fn window_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Liquid".to_owned(),
        window_width: 1280,
        window_height: 800,
        window_resizable: true,
        ..Default::default()
    }
}
