use crate::Vulnerability;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::io;

pub fn run_vulnerability_dashboard(vulns: Vec<Vulnerability>) -> Result<()> {
    if vulns.is_empty() {
        println!("\n[CERBERUS SECURITY REVIEW]\n✅ No vulnerabilities found!");
        return Ok(());
    }

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new(vulns);
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

struct App {
    vulns: Vec<Vulnerability>,
    state: ListState,
}

impl App {
    fn new(vulns: Vec<Vulnerability>) -> App {
        let mut state = ListState::default();
        state.select(Some(0));
        App { vulns, state }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.vulns.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.vulns.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                _ => {}
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.area());

    // Left Panel: List of vulnerabilities
    let items: Vec<ListItem> = app
        .vulns
        .iter()
        .map(|v| {
            let color = match v.severity.to_lowercase().as_str() {
                "critical" => Color::LightRed,
                "high" => Color::Red,
                "medium" => Color::Yellow,
                "low" => Color::Green,
                _ => Color::White,
            };
            let content = vec![Line::from(vec![
                Span::styled(format!("[{}] ", v.severity.to_uppercase()), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(v.file.clone()),
            ])];
            ListItem::new(content)
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Vulnerabilities"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, chunks[0], &mut app.state);

    // Right Panel: Details
    if let Some(i) = app.state.selected() {
        let vuln = &app.vulns[i];
        
        let text = vec![
            Line::from(Span::styled("Description", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))),
            Line::from(vuln.description.clone()),
            Line::from(""),
            Line::from(Span::styled("Remediation", Style::default().add_modifier(Modifier::BOLD).fg(Color::Green))),
            Line::from(vuln.remediation.clone()),
            Line::from(""),
            Line::from(Span::styled("Press 'q' or 'Esc' to exit", Style::default().fg(Color::DarkGray))),
        ];

        let block = Block::default()
            .title(format!(" {} ", vuln.file))
            .borders(Borders::ALL);
        
        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, chunks[1]);
    }
}
