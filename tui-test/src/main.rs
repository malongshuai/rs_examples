use crossterm::{
    event::{
        poll, read, DisableFocusChange, EnableFocusChange, Event,
        KeyCode::{self, *},
        KeyEventKind, KeyModifiers, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnableLineWrap},
    ExecutableCommand, Result,
};
use std::{
    io::{self, Write},
    time,
};

fn main() -> Result<()> {
    let timeout = time::Duration::from_secs(10);
    enable_raw_mode()?;

    while poll(timeout)? {
        if let Event::Key(kv) = read()? {
            // let kc = kv.code;
            // let k_kind = kv.kind;
            // let k_m = kv.modifiers;
            // if k_kind == KeyEventKind::Press {
            //     if k_m == KeyModifiers::CONTROL && kc == KeyCode::Char('c') {
            //         print!("GoodBye\r\n");
            //         break;
            //     }
            //     match kc {
            //         Char(c) => print!("{}", c),
            //         Enter => print!("\r\n"),
            //         Backspace => print!("\x08 \x08"),
            //         _ => print!("OtherKey: {:?}", kc),
            //     };
            //     io::stdout().flush()?;
            // }
            print!("{:?}, {:?}\r\n", kv, kv.code);
        }
    }

    disable_raw_mode()?;

    Ok(())
}
