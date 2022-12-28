use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::DP2200::datapoint::{self, DataPointRunStatus, Datapoint};
use crate::DP2200::disassembler::Disassembler;
use crate::DP2200::instruction::{FLAG_NAME, REG_NAME};
use log::{info, trace, warn};
use wasm_bindgen::{closure::WasmClosure, prelude::*, JsCast};
use web_sys::{
    window, Document, Element, Event, HtmlButtonElement, HtmlTableCellElement, HtmlTableRowElement,
    KeyboardEvent,
};

#[derive(Debug)]
pub struct UiState {
    pub document: Document,
    pub datapoint: Datapoint,
    pub disassembler: Disassembler,
    pub disassembler_table_rows: Vec<String>,
    pub running: bool,
    pub emulation_closure: Closure<dyn Fn(f64)>,
    last_animation_frame: f64,
}

#[derive(Debug, Clone)]
pub struct State {
    pub ui_state: Rc<RefCell<UiState>>,
}

impl State {
    pub fn new() -> State {
        let ui_state = Rc::new(RefCell::new(UiState::new()));
        let state = State { ui_state };
        state.init_single_step_button();
        state.init_run_button();
        state.init_mem_table();
        state.init_pause_button();
        state.init_emulation_closure();

        state.init_keyboard_events();
        state
    }

    fn init_keyboard_events(&self) {
        let ui_state = self.ui_state.clone();
        let keydown_handler: Closure<dyn Fn(_)> = Closure::new(move |event: KeyboardEvent| {
            ui_state
                .borrow_mut()
                .datapoint
                .databus
                .keyboard
                .keydown(event.key())
        });

        let ui_state = self.ui_state.clone();
        let keyup_handler: Closure<dyn Fn(_)> = Closure::new(move |event: KeyboardEvent| {
            ui_state
                .borrow_mut()
                .datapoint
                .databus
                .keyboard
                .keyup(event.key())
        });
        self.ui_state
            .borrow()
            .document
            .add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref());

        self.ui_state
            .borrow()
            .document
            .add_event_listener_with_callback("keyup", keyup_handler.as_ref().unchecked_ref());

        keydown_handler.forget();
        keyup_handler.forget();
    }

    fn init_emulation_closure(&self) {
        let ui_state = self.ui_state.clone();
        self.ui_state.borrow_mut().emulation_closure = Closure::new(move |time_stamp: f64| {
            if !ui_state.borrow().running {
                ui_state.borrow_mut().draw();
                return;
            }

            let delta_time_ms = time_stamp - ui_state.borrow().last_animation_frame;
            ui_state.borrow_mut().last_animation_frame = time_stamp;
            let emulation_result = ui_state.borrow_mut().datapoint.update(delta_time_ms);

            trace!("{:?}", emulation_result);
            if emulation_result == DataPointRunStatus::Ok {
                ui_state.borrow().request_animation_frame();
                ui_state.borrow().draw_screen();
            } else {
                ui_state.borrow_mut().running = false;
                ui_state.borrow_mut().draw();
            }
        });
    }

    fn init_pause_button(&self) {
        let ui_state = self.ui_state.clone();
        let pause_closure = Closure::<dyn Fn(_)>::new(move |_event: Event| {
            ui_state.borrow_mut().running = false;
        });

        self.add_event("pause", "click", pause_closure);
    }

    fn init_single_step_button(&self) {
        let ui_state = self.ui_state.clone();
        let single_step_closure = Closure::<dyn Fn(_)>::new(move |_event: Event| {
            ui_state.borrow_mut().datapoint.single_step();
            ui_state.borrow_mut().draw();
            ui_state.borrow().focus_row();
        });

        self.add_event("single_step", "click", single_step_closure);
    }

    fn init_mem_table(&self) {
        let ui_state = self.ui_state.clone();
        let table_click_closure = Closure::<dyn Fn(_)>::new(move |event: Event| {
            if let Ok(cell) = event.target().unwrap().dyn_into::<HtmlTableCellElement>() {
                if let Ok(row) = cell
                    .parent_element()
                    .expect("No parent found")
                    .dyn_into::<HtmlTableRowElement>()
                {
                    let mut breakpoint_addr = None;
                    if row.row_index() != -1 {
                        if let Some((addr, _)) = ui_state
                            .borrow()
                            .disassembler
                            .addr_to_line
                            .get((row.row_index() - 1) as usize)
                        {
                            breakpoint_addr = Some(*addr);
                        }
                    }

                    if let Some(addr) = breakpoint_addr {
                        ui_state.borrow_mut().datapoint.toggle_breakpoint(addr);

                        ui_state.borrow_mut().draw();
                    }
                }
            }
        });
        self.add_event("mem_table", "click", table_click_closure);
    }

    fn init_run_button(&self) {
        let ui_state = self.ui_state.clone();
        let run_closure = Closure::<dyn Fn(_)>::new(move |_event: Event| {
            ui_state.borrow_mut().last_animation_frame =
                window().unwrap().performance().unwrap().now();
            ui_state.borrow_mut().running = true;
            ui_state.borrow_mut().draw();
            ui_state.borrow().request_animation_frame();
        });
        self.add_event("run", "click", run_closure);
    }

    pub fn draw(&self) {
        self.ui_state.borrow_mut().draw();
        self.draw_disassembler();
    }

    pub fn get_element_by_id(&self, id: &str) -> Element {
        self.ui_state
            .borrow()
            .document
            .get_element_by_id(id)
            .expect("No element found")
    }
    pub fn add_event(&self, id: &str, event_type: &str, function: Closure<dyn Fn(Event)>) {
        let element = self.get_element_by_id(id);
        element.add_event_listener_with_callback(event_type, function.as_ref().unchecked_ref());
        function.forget();
    }

    pub fn load_program(&self, program: Vec<u8>) {
        self.ui_state.borrow_mut().running = false;
        self.ui_state.borrow_mut().datapoint.load_program(&program);
        self.ui_state.borrow_mut().disassembler = Disassembler::from_vec(&program);
    }

    pub fn draw_disassembler(&self) {
        self.ui_state.borrow_mut().draw_disassembler();
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl UiState {
    pub fn new() -> UiState {
        let window = web_sys::window().expect("no global `window` exists");
        let datapoint = Datapoint::build(&Vec::new(), 1.0);

        let emulation_closure = Closure::<dyn Fn(f64)>::new(|_: f64| {});

        UiState {
            document: window.document().expect("No document found"),
            running: false,
            datapoint,
            disassembler: Disassembler::from_vec(&Vec::new()),
            disassembler_table_rows: Vec::new(),
            emulation_closure: emulation_closure,
            last_animation_frame: 0.0,
        }
    }

    fn update_element_by_id(&self, id: &str, new_content: &str) {
        let el = self
            .document
            .get_element_by_id(id)
            .expect("Element not found");
        el.set_inner_html(new_content);
    }

    fn disable_button(&self, id: &str, disable: bool) {
        if let Ok(el) = self
            .document
            .get_element_by_id(id)
            .expect("Element not found")
            .dyn_into::<HtmlButtonElement>()
        {
            el.set_disabled(disable);
        }
    }

    pub fn request_animation_frame(&self) {
        web_sys::window()
            .unwrap()
            .request_animation_frame(self.emulation_closure.as_ref().unchecked_ref())
            .unwrap();
    }

    pub fn draw_stack(&self) {
        let mut table = String::new();
        for i in 0..self.datapoint.cpu.stack.len() {
            table.push_str(
                format!(
                    "<tr><td>{}</td><td>{:#06x}</td></tr>",
                    i, self.datapoint.cpu.stack[i]
                )
                .as_str(),
            );
        }

        self.update_element_by_id("stack_table", table.as_str());
    }

    pub fn draw_screen(&self) {
        let canvas = self.document.get_element_by_id("screen_canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
        context.set_fill_style(&"white".into());
        context.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        context.set_fill_style(&"black".into());
        context.set_font("20px Monospace");
        for l in 0..self.datapoint.databus.screen.buffer.len() {
            let s: String = self.datapoint.databus.screen.buffer[l].iter().collect();

            context
                .fill_text(s.as_str(), 2.0, 20.0 * (l + 1) as f64)
                .unwrap();
        }
    }

    pub fn draw_cpu_status(&self) {
        let mut table = String::new();
        for i in 0..7 {
            table.push_str(
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                    REG_NAME[i],
                    self.datapoint.cpu.alpha_registers[i],
                    self.datapoint.cpu.beta_registers[i]
                )
                .as_str(),
            );
        }

        self.update_element_by_id("register_table", table.as_str());

        let mut table = String::new();
        for i in 0..3 {
            table.push_str(
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                    FLAG_NAME[i],
                    self.datapoint.cpu.alpha_flipflops[i],
                    self.datapoint.cpu.beta_flipflops[i]
                )
                .as_str(),
            );
        }

        self.update_element_by_id("flag_table", table.as_str());

        self.draw_stack();
    }

    pub fn draw(&mut self) {
        if self.running {
            self.disable_button("single_step", true);
            self.disable_button("run", true);
            self.disable_button("pause", false);
        } else {
            self.disable_button("single_step", false);
            self.disable_button("run", false);
            self.disable_button("pause", true);
            self.draw_cpu_status();
            self.draw_disassembler();
        }

        self.draw_screen();
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}
