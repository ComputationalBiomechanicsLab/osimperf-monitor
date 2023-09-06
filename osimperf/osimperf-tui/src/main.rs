use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use osimperf_lib::{
    bench_tests::{BenchTestResult, BenchTestSetup},
    Archive, CompilationNode, Complete, Focus, Folder, Home, Progress, ResultsFolder, Status,
};
use ratatui::{prelude::*, widgets::*};
use std::{
    error::Error,
    io::{self, Stdout},
    path::PathBuf,
    time::Duration,
};

#[derive(Parser, Debug)]
pub struct Args {
    /// Specify path to osimperf home dir. If not, current directory will be used as home.
    #[arg(long)]
    pub home: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut terminal = setup_terminal()?;

    // create app and run it
    let app = App::new(&args)?;
    let res = run_app(&mut terminal, app);

    // Shutting down program.
    restore_terminal(&mut terminal)?;
    if let Err(err) = res {
        println!("{err:?}");
    }
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    Ok(terminal.show_cursor()?)
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        let mut output = Ok(());
        terminal.draw(|f| wrap_ui(f, &mut app, &mut output))?;
        output?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn wrap_ui<B: Backend>(f: &mut Frame<B>, app: &mut App, result: &mut Result<()>) {
    *result = ui(f, app);
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) -> Result<()> {
    let nodes = CompilationNode::collect_archived(&app.archive)?;
    let tests = BenchTestSetup::find_all(&app.tests_dir)?;

    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    const node_cols: usize = 3;
    let bench_cols = tests.len();

    let normal_style = Style::default().bg(Color::Blue);

    let mut widths: Vec<Constraint> = vec![
        Constraint::Length(15),
        Constraint::Length(10),
        Constraint::Length(15),
    ];
    widths.extend((0..bench_cols).map(|_| Constraint::Length(20)));

    let mut compiled_size = 0;
    let mut compiled_duration = 0;

    let mut tests_duration = vec![0; bench_cols];

    let mut rows: Vec<Row> = Vec::new();
    for node in nodes.iter() {
        let mut cells: Vec<Cell> = Vec::new();

        cells.push(Cell::from(node.repo.name.as_str()));
        cells.push(Cell::from(node.repo.date.as_str()));
        for (i, state) in node
            .state
            .get()
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_done())
        {
            let focus = Focus::from(i);
            cells.push(match state {
                Status::Idle => Cell::from(format!("Queued {}", focus.short_desc())),
                Status::Compiling(Progress { percentage }) => {
                    Cell::from(format!("{}: {}%", focus.short_desc(), percentage))
                        .set_style(Style::default().bg(Color::Blue))
                }
                Status::Error(_) => Cell::from(format!("{}: Failed", focus.short_desc()))
                    .set_style(Style::default().bg(Color::Red)),
                _ => panic!(),
            });
            break;
        }
        if node.state.get().iter().all(|s| s.is_done()) {
            cells.push(match node.state.get()[1] {
                Status::Done(Complete { duration, size }) => {
                    compiled_size += size;
                    compiled_duration += duration.as_secs() / 60;
                    Cell::from("Done").set_style(Style::default().bg(Color::Green))
                }
                _ => panic!(),
            });
        }

        // cells.push(Cell::from(node.state.get()[1].print_table_entry()));

        // Print a column for each test.
        if !node.is_done() {
            for _ in tests.iter() {
                cells.push(Cell::from(" "));
            }
        } else {
            for (i, t) in tests.iter().enumerate() {
                let result = BenchTestResult::read(&app.results_dir, &node.id(), &t.name)?;
                cells.push(
                    match (
                        result.as_ref().and_then(|x| x.duration),
                        result
                            .as_ref()
                            .and_then(|x| x.duration_stddev)
                            .unwrap_or(f64::NAN),
                        result.as_ref().map(|x| x.iteration),
                        result.as_ref().map(|x| x.failed_count),
                    ) {
                        (_, _, _, Some(i)) if i > 0 => {
                            Cell::from("Failed").style(Style::default().fg(Color::Red))
                        }
                        (Some(dt), stddev, Some(_), _) if stddev < 1e-2 => {
                            Cell::from(format!("{:.2}", dt))
                        }
                        (Some(dt), stddev, Some(iter), _) => {
                            Cell::from(format!("{:.2} ({:.3}, {iter}X)", dt, stddev))
                                .style(Style::default().fg(Color::DarkGray))
                        }
                        _ => Cell::from("Queued"),
                    },
                );
            }
        }
        rows.push(Row::new(cells));
    }

    let mut header_cells = vec![
        Cell::from("Version"),
        Cell::from("Date"),
        Cell::from(format!(
            "Status\n{}Gb {}min",
            compiled_size / 1000, compiled_duration
        )),
    ];

    // Header:
    header_cells.extend(
        tests
            .iter()
            .map(|t| &t.name)
            .map(|h| Cell::from(h.as_str()).style(Style::default().bg(Color::DarkGray))),
    );
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    // Start building the table.
    let t = Table::new(rows)
        // You can set the style of the entire Table.
        .style(Style::default().fg(Color::White))
        // It has an optional header, which is simply a Row always visible at the top.
        .header(header)
        // As any other widget, a Table can be wrapped in a Block.
        .block(Block::default().borders(Borders::ALL).title("Table"))
        // Columns widths are constrained in the same way as Layout...
        .widths(&widths)
        // ...and they can be separated by a fixed spacing.
        .column_spacing(1)
        // If you wish to highlight a row in any specific way when it is selected...
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        // ...and potentially show a symbol in front of the selection.
        .highlight_symbol(">>");

    f.render_widget(t, rects[0]);

    Ok(())
}

struct App {
    archive: Archive,
    results_dir: ResultsFolder,
    tests_dir: PathBuf,
}

impl App {
    fn new(args: &Args) -> Result<App> {
        let home = Home::new_or_current(args.home.as_ref().map(|p| p.as_str()))?;
        Ok(App {
            archive: home.default_archive()?,
            results_dir: home.default_results()?,
            tests_dir: home.path()?.join("tests"),
        })
    }
}
