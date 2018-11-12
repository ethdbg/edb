mod events;
mod tabs;

use std::{
    io::{self, Write},
    time::{Duration, Instant}
};
use termion::{
    input::MouseTerminal,
    event::Key,
    raw::IntoRawMode,
    screen::AlternateScreen,
    cursor::Goto};
use tui::{
    Terminal,
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Rect, Corner},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget, Tabs, Text, List, Paragraph}
};
use unicode_width::UnicodeWidthStr;

use self::{
    events::{Event, Events},
    tabs::TabsState
};

pub const MAX_FRAMES: u128 = 125;

struct App<'a> {
    size: Rect,
    tabs: TabsState<'a>,
    // Current Value of Input
    input: String,
    // History of recorded messages
    messages: Vec<String>
}

pub fn launch_tui() -> Result<(), failure::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Event Handlers
    let events = Events::new();

    // default app state
    let mut app = App {
        size: Rect::default(),
        tabs: TabsState::new(vec!["Debug", "Memory", "Stack", "Storage"]),
        input: String::new(),
        messages: Vec::new()
    };

    'tui_main: loop {
        let now = Instant::now();
        let size = terminal.size()?;
        if app.size != size {
            terminal.resize(size)?;
            app.size = size;
        }

        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(70),
                        Constraint::Percentage(20),
                    ].as_ref())
                .split(app.size);
            Tabs::default()
                .block(Block::default().borders(Borders::ALL).title("Tabs"))
                .titles(&app.tabs.titles)
                .select(app.tabs.index)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(Style::default().fg(Color::Yellow))
                .render(&mut f, chunks[0]);
            match app.tabs.index {
                0 => Block::default()
                    .borders(Borders::NONE)
                    .render(&mut f, chunks[1]),
                1 => Block::default()
                    .borders(Borders::NONE)
                    .render(&mut f, chunks[1]),
                2 => Block::default()
                    .borders(Borders::NONE)
                    .render(&mut f, chunks[1]),
                3 => Block::default()
                    .borders(Borders::NONE)
                    .render(&mut f, chunks[1]),
                _ => {}
            }
            { // console
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(15)].as_ref())
                    .split(chunks[2]);
                let messages = app
                    .messages
                    .iter()
                    .map(|m| Text::raw(format!(">> {}",m)));
                Paragraph::new([Text::raw(&app.input)].iter())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title("Input"))
                    .render(&mut f, chunks[0]);
                List::new(messages)
                    .block(Block::default().borders(Borders::NONE).title("Console"))
                    .start_corner(Corner::BottomLeft)
                    .render(&mut f, chunks[1]);

            }
        })?;
        write!(
            terminal.backend_mut(),
            "{}",
            Goto(4 + app.input.width() as u16, 4)
        )?;
        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q')  => break,
                Key::Char('1') => app.tabs.tab(0),
                Key::Char('2') => app.tabs.tab(1),
                Key::Char('3') => app.tabs.tab(2),
                Key::Char('4') => app.tabs.tab(3),
                Key::Char('\n') => {
                    app.messages.push(app.input.drain(..).collect());
                },
                Key::Char(c) => app.input.push(c),
                Key::Backspace => {
                    app.input.pop();
                },
                _ => {}
            },
            _ => {}
        }
        if app.messages.len() > 5 {
            app.messages.reverse();
            app.messages.pop();
            app.messages.reverse();
        }
        // frame limiting
        if now.elapsed().as_micros() < (MAX_FRAMES / 1_000_000) {
            std::thread::sleep(Duration::from_micros( ((MAX_FRAMES / 1_000_000) - now.elapsed().as_micros()) as u64))
        }
    }
    Ok(())
}

// sendTransaction
/*
Object - The transaction object to send:

    from - String|Number: The address for the sending account. Uses the web3.eth.defaultAccount property, if not specified. Or an address or index of a local wallet in web3.eth.accounts.wallet.
    to - String: (optional) The destination address of the message, left undefined for a contract-creation transaction.
    value - Number|String|BN|BigNumber: (optional) The value transferred for the transaction in wei, also the endowment if it’s a contract-creation transaction.
    gas - Number: (optional, default: To-Be-Determined) The amount of gas to use for the transaction (unused gas is refunded).
    gasPrice - Number|String|BN|BigNumber: (optional) The price of gas for this transaction in wei, defaults to web3.eth.gasPrice.
    data - String: (optional) Either a ABI byte string containing the data of the function call on a contract, or in the case of a contract-creation transaction the initialisation code.
    nonce - Number: (optional) Integer of a nonce. This allows to overwrite your own pending transactions that use the same nonce.
*/

