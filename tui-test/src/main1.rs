use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Stdout},
    thread,
    time::Duration,
};
use tui::{
    backend::Backend,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame, Terminal,
};

type TerminalInstance = Terminal<CrosstermBackend<Stdout>>;

// 创建并进入新的终端实例的操作：创建并控制新终端
fn set_up_terminal() -> Result<TerminalInstance, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

// 恢复到原有终端时的操作：离开控制终端回到原有终端，不再捕获鼠标事件
fn restore_terminal(mut terminal: TerminalInstance) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()
}

// 界面要绘制的内容：以Frame为单位进行绘制，可以按需来布局Frame，
// 例如此处将该Frame水平划分为上中下三块区域，大小各占10% 80% 10%
fn ui<B: Backend>(f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());
    let block = Block::default().title("Block").borders(Borders::ALL);
    f.render_widget(block, chunks[0]);
    let block = Block::default().title("Block 2").borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
    let block = Block::default().title("Block 3").borders(Borders::ALL);
    f.render_widget(block, chunks[2]);
}

// fn main() -> Result<(), io::Error> {
// // 生成一个新的终端实例并取得它的控制权
// let mut terminal = set_up_terminal()?;

// // 将内容绘制到终端界面
// terminal.draw(|f| { ui(f); })?;

// thread::sleep(Duration::from_millis(5000));

// // 恢复到原有的终端
// restore_terminal(terminal)?;

// Ok(())

// }

// use std::io::{stdout, Write};
// use crossterm::{
//     ExecutableCommand, QueueableCommand,
//     terminal, cursor, style::{self, Stylize}
// };

// fn main() -> Result<(), io::Error> {
//   let mut stdout = stdout();

//   stdout.execute(terminal::Clear(terminal::ClearType::All))?;

//   for y in 0..40 {
//     for x in 0..150 {
//       if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
//         // in this loop we are more efficient by not flushing the buffer.
//         stdout
//           .queue(cursor::MoveTo(x,y))?
//           .queue(style::PrintStyledContent( "█".magenta()))?;
//       }
//     }
//   }
//   stdout.flush()?;
//   Ok(())
// }
