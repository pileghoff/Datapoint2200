use core::panic;
use glob::glob;
use std::{
    env,
    fs::{read, remove_file, write, File},
    io::{self, Read, Write},
    path::Path,
};


use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::Stylize,
    widgets::Paragraph,
    DefaultTerminal,
};
use std::time::Duration;
pub mod DP2200;
use DP2200::datapoint;
// fn main() {
//         let data = read(path).unwrap();
//         let mut machine = datapoint::Datapoint::build(&data, 1.0);
//         machine.load_cassette(data);
//         while !machine.cpu.halted {
//             println!("Runnig");
//             machine.update(10.0);
//             println!("{}", machine.databus.screen.get_screen());
//         }
//     }
// }

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal);
    ratatui::restore();
    app_result
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let path = args.get(1).unwrap();
    let data = read(path).unwrap();

    let mut machine = datapoint::Datapoint::build(&data, 1.0);
    machine.load_cassette(data);


    let mut i = 0;
    while !machine.cpu.halted {
        machine.update(100.0);
        let mut key_msg = String::new();

        if event::poll(Duration::from_millis(100)).unwrap() {
            if let event::Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc {
                    break;
                }
                if key.kind == KeyEventKind::Press {
                    machine.databus.keyboard.keydown(key.code.to_string());
                    machine.update(10.0);
                    machine.databus.keyboard.keyup(key.code.to_string());
                    key_msg = format!("Key: {} pressed", key.code.to_string());
                }
                if key.kind == KeyEventKind::Release {
                    key_msg = format!("Key: {} pressed", key.code.to_string());
                }
            }
        }

        terminal.draw(|frame| {
            let greeting = Paragraph::new(format!("Greetings: {}\n{}", machine.databus.screen.get_screen(), key_msg))
                .white();
            frame.render_widget(greeting, frame.area());
        })?;
    }

    Ok(())
}