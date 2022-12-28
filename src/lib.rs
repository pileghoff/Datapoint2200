use std::ops::Deref;

use log::Level;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{console, Document};
pub mod DP2200;
mod time;
pub mod ui;
use ui::ui::*;
use DP2200::assembler::assemble;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug);
    let program = include_str!("../test_software/display.asm");
    let program = assemble(program.lines().collect()).unwrap();
    //let program = include_bytes!("../test_software/test.bin").to_vec();
    let ui = State::new();
    ui.load_program(program);
    ui.draw();

    Ok(())
}
