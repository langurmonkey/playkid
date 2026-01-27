#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod apu;
mod cartridge;
mod cli;
mod constants;
mod debugmanager;
mod eventhandler;
mod gui;
mod instruction;
mod joypad;
mod machine;
mod memory;
mod playkid;
mod ppu;
mod registers;
mod timer;
mod uistate;

use clap::Parser;
use cli::Args;
use constants::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use playkid::PlayKid;

use eframe::egui;
use eframe::egui::Visuals;

// Native compilation.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();
    let args = Args::parse();

    let mut window_width = DISPLAY_WIDTH as f32 * args.scale as f32;
    let window_height = DISPLAY_HEIGHT as f32 * args.scale as f32;

    // If debugging is enabled, add extra width for the SidePanel.
    if args.debug {
        window_width += 450.0;
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([window_width, window_height])
            .with_min_inner_size([DISPLAY_WIDTH as f32, DISPLAY_HEIGHT as f32])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../img/logo.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        constants::NAME,
        native_options,
        Box::new(|cc| {
            // Set dark mode by default
            cc.egui_ctx.set_visuals(Visuals::dark());

            Ok(Box::new(PlayKid::new(cc, args)))
        }),
    )
}

// Web compilation.
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("playkid_canvas")
            .expect("Failed to find playkid_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("playkid_canvas was not a HtmlCanvasElement");

        // Create arguments.

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    Ok(Box::new(eframe_template::EmulatorApp::new_wasm(
                        cc,
                        "path/to/rom.gb",
                    )))
                }),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
