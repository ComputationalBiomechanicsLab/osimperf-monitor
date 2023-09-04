use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use osimperf_lib::{
    bench_tests::{BenchTestResult, BenchTestSetup},
    Archive, CompilationNode, Complete, Folder, Home, Progress, ResultsFolder, Status,
};
use ratatui::{prelude::*, widgets::*};
use std::{
    error::Error,
    io::{self, Stdout}, path::PathBuf,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;

    // create app and run it
    let app = App::new()?;
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

fn wrap_ui<B: Backend>(f: &mut Frame<B>, app: &mut App, result: &mut Result<()>) {
    *result = ui(f, app);
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) -> Result<()> {
    let nodes = CompilationNode::collect_archived(&app.archive)?;
    let tests = BenchTestSetup::find_all(&app.tests_dir)?;

    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let normal_style = Style::default().bg(Color::Blue);

    // Header:
    // Err(anyhow!(format!("node is done found = {:?}", header_cells)))?;
    let header_cells = app
        .node_col_headers
        .iter()
        .map(|h| Cell::from(h.as_str()))
        .chain(
            tests
                .iter()
                .map(|t| &t.name)
                .map(|h| Cell::from(h.as_str()).style(Style::default().bg(Color::Gray))),
        );
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    let ncols = app.node_col_headers.len() + tests.len();

    let widths: Vec<Constraint> = (0..ncols).map(|_| Constraint::Length(20)).collect();

    let mut rows: Vec<Row> = Vec::new();
    for node in nodes.iter() {
        let mut cells: Vec<Cell> = Vec::new();

        cells.push(Cell::from(format!("{}-{}", node.repo.name, node.repo.date)));
        cells.push(match node.state.get()[1] {
            Status::Idle => Cell::from("Queued"),
            Status::Compiling(Progress { percentage }) => {
                Cell::from(format!("{}%", percentage)).set_style(Style::default().bg(Color::Blue))
            }
            Status::Error(_) => Cell::from("Failed").set_style(Style::default().bg(Color::Red)),
            Status::Done(Complete { duration, size }) => Cell::from(format!(
                "Done ({} min, {} Gb)",
                duration.as_secs() / 60,
                size
            ))
            .set_style(Style::default().bg(Color::Green)),
        });
        // cells.push(Cell::from(node.state.get()[1].print_table_entry()));

        // Print a column for each test.
        if !node.is_done() {
            for _ in tests.iter() {
                cells.push(Cell::from(" "));
            }
        } else {
            for t in tests.iter() {
                let result = BenchTestResult::read(&app.results_dir, &node.id(), &t.name)?;
                if let (Some(dt), stddev, Some(iter)) = (
                    result.as_ref().and_then(|x| x.duration),
                    result
                        .as_ref()
                        .and_then(|x| x.duration_stddev)
                        .unwrap_or(f64::NAN),
                    result.as_ref().map(|x| x.iteration),
                ) {
                    cells.push(if stddev < 1e-2 {
                        Cell::from(format!("{:.2}", dt))
                    } else {
                        Cell::from(format!("{:.2} ({:.3}, {iter}X)", dt, stddev))
                            .style(Style::default().fg(Color::Red))
                    });
                }
            }
        }
        rows.push(Row::new(cells));
    }

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
    node_col_headers: [String; 2],
}

impl App {
    fn new() -> Result<App> {
        let home = Home::new_or_current(None)?;
        Ok(App {
            archive: home.default_archive()?,
            results_dir: home.default_results()?,
            tests_dir: home.path()?.join("tests"),
            node_col_headers: ["Version".to_string(), "Status".to_string()],
        })
    }
}

// fn make_row<'a>(
//     node: &CompilationNode,
//     tests: &[BenchTestSetup],
//     results: &ResultsFolder,
// ) -> Result<Row<'a>> {
//     let mut cells: Vec<Cell> = Vec::new();

//     cells.push(Cell::from(node.repo.name.as_str()));
//     cells.push(Cell::from(node.repo.date.as_str()));
//     cells.push(Cell::from(node.state.get()[1].print_table_entry()));

//     // Print a column for each test.
//     if !node.is_done() {
//         for _ in tests.iter() {
//             cells.push(Cell::from(" "));
//         }
//     } else {
//         for t in tests.iter() {
//             let result = BenchTestResult::read(results, &node.id(), &t.name)?;
//             if let (Some(dt), stddev, Some(iter)) = (
//                 result.as_ref().and_then(|x| x.duration),
//                 result
//                     .as_ref()
//                     .and_then(|x| x.duration_stddev)
//                     .unwrap_or(f64::NAN),
//                 result.as_ref().map(|x| x.iteration),
//             ) {
//                 cells.push(Cell::from(if stddev < 1e-2 {
//                     format!("{:.2}", dt)
//                 } else {
//                     format!("{:.2} ({:.3}, {iter}X)", dt, stddev)
//                 }));
//             }
//         }
//     }

//     Ok(Row::new(cells))

//     // vec![

//     // // Row can be created from simple strings.
//     // Row::new(vec!["Row11", "Row12", "Row13"]),
//     // // You can style the entire row.
//     // Row::new(vec!["Row21", "Row22", "Row23"]).style(Style::default().fg(Color::Blue)),
//     // // If you need more control over the styling you may need to create Cells directly
//     // Row::new(vec![
//     //     Cell::from("Row31"),
//     //     Cell::from("Row32").style(Style::default().fg(Color::Yellow)),
//     //     Cell::from(Line::from(vec![
//     //         Span::raw("Row"),
//     //         Span::styled("33", Style::default().fg(Color::Green)),
//     //     ])),
//     // ]),
//     // // If a Row need to display some content over multiple lines, you just have to change
//     // // its height.
//     // Row::new(vec![
//     //     Cell::from("Row\n41"),
//     //     Cell::from("Row\n42"),
//     //     Cell::from("Row\n43"),
//     // ])
//     // .height(2),
//     // ])
// }
