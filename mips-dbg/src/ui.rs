//! TUI interface for the debugger

use crate::debugger::Debugger;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

pub fn start_tui(debugger: &mut Debugger) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let res = run_app(&mut terminal, debugger);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, debugger: &mut Debugger) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, debugger))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('s') => {
                    // Step instruction
                    let _ = debugger.step();
                }
                KeyCode::Char('c') => {
                    // Continue execution
                    let _ = debugger.run();
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, debugger: &Debugger) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.area());

    // Registers panel
    let registers = debugger.get_registers();
    let register_items: Vec<ListItem> = (0..32)
        .map(|i| {
            ListItem::new(Line::from(Span::raw(format!(
                "${:2}: 0x{:08x}",
                i, registers[i]
            ))))
        })
        .collect();

    let register_list = List::new(register_items)
        .block(Block::default().title("Registers").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(register_list, chunks[0]);

    // Main content area
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(chunks[1]);

    // Program counter and instruction view
    let pc_text = Paragraph::new(format!("PC: 0x{:08x}", debugger.get_pc())).block(
        Block::default()
            .title("Program Counter")
            .borders(Borders::ALL),
    );

    f.render_widget(pc_text, main_chunks[0]);

    // Command help
    let help_text = Paragraph::new(
        "Commands:\n\
         q - Quit\n\
         s - Step instruction\n\
         c - Continue execution",
    )
    .block(Block::default().title("Help").borders(Borders::ALL));

    f.render_widget(help_text, main_chunks[1]);
}
