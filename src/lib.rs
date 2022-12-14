use log::Level;
use wasm_bindgen::prelude::*;
pub mod DP2200;
mod time;
pub mod ui;
use ui::ui::*;
use DP2200::assembler::assemble;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).unwrap();
    //let program = include_str!("../test_software/display.asm");
    //let program = assemble(program.lines().collect()).unwrap();
    //let program = include_bytes!("../test_software/dosAbootVer2.tap").to_vec();
    let program = include_bytes!("../test_software/db2intv4.1_8-74.tap").to_vec();
    let ui = State::new();
    ui.load_program(program);
    ui.draw();

    Ok(())
}
