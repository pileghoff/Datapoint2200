use log::info;
use wasm_bindgen::JsCast;
use web_sys::{
    Document, HtmlElement, HtmlTableCellElement, HtmlTableElement, HtmlTableRowElement,
    ScrollBehavior, ScrollIntoViewOptions, ScrollLogicalPosition,
};

use super::ui::UiState;

impl UiState {
    pub fn focus_row(&self) {
        if let Some(row) = self.document.get_element_by_id("focus_row") {
            if let Ok(row) = row.dyn_into::<HtmlElement>() {
                let mut scroll_option = ScrollIntoViewOptions::new();
                scroll_option.behavior(ScrollBehavior::Smooth);
                scroll_option.block(ScrollLogicalPosition::Nearest);
                row.scroll_into_view_with_scroll_into_view_options(&scroll_option);
            }
        }
    }

    fn construct_row(&self, addr: u16, text: &str) -> String {
        let mut current_address_arrow = String::new();
        let mut row_id = String::new();
        let mut breakpoint_arrow = String::new();
        if !self.running && self.datapoint.cpu.program_counter == addr {
            current_address_arrow.push('>');
            row_id.push_str("id=\"focus_row\"");
        }
        if self.datapoint.breakpoints.contains(&addr) {
            breakpoint_arrow.push('*');
        }
        format!(
            "<tr {}><td>{}</td><td>{}</td><td>{:#06x}</td><td>{}</td></tr>",
            row_id, breakpoint_arrow, current_address_arrow, addr, text
        )
    }

    pub fn draw_disassembler(&mut self) {
        let table = self
            .document
            .get_element_by_id("mem_table")
            .expect("No memtable found")
            .dyn_into::<HtmlElement>()
            .unwrap();

        for (index, (addr, text)) in self.disassembler.addr_to_line.iter().enumerate() {
            let row_str = self.construct_row(*addr, text);
            if let Some(row_cached) = self.disassembler_table_rows.get_mut(index) {
                if *row_cached == row_str {
                    continue;
                }

                row_cached.replace_range(.., &row_str);
            } else {
                self.disassembler_table_rows.push(row_str.clone());
            }

            if let Some(child_to_replace) = table.children().get_with_index(index as u32) {
                child_to_replace.set_outer_html(&row_str);
            } else {
                table.insert_adjacent_html("beforeend", &row_str).unwrap();
            }
        }
    }
}
