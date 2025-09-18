use std::io::{Result, stdout};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    prelude::{Alignment, Constraint, CrosstermBackend, Direction, Layout, Terminal},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
};

use sysinfo::{Disks, ProcessRefreshKind, ProcessesToUpdate, System};

// System struct to hold system information
struct App {
    system: System,
    processes_state: TableState,
    tick_count: u64,       // Add a counter to track refresh cycles
    disks: sysinfo::Disks, // Add disks to track disk usage
}

impl App {
    fn new() -> Self {
        Self {
            system: System::new_all(),
            processes_state: TableState::default(),
            tick_count: 0,
            disks: sysinfo::Disks::new_with_refreshed_list(),
        }
    }

    fn on_tick(&mut self) {
        self.tick_count += 1;
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );
        // Disk refresh is handled automatically
    }
}

/* Sysmon - CLI */

const ASCII_ART: &str = r#"
███████╗██╗   ██╗███████╗███╗   ███╗ ██████╗ ███╗   ██╗
██╔════╝╚██╗ ██╔╝██╔════╝████╗ ████║██╔═══██╗████╗  ██║
███████╗ ╚████╔╝ ███████╗██╔████╔██║██║   ██║██╔██╗ ██║
╚════██║  ╚██╔╝  ╚════██║██║╚██╔╝██║██║   ██║██║╚██╗██║
███████║   ██║   ███████║██║ ╚═╝ ██║╚██████╔╝██║ ╚████║
╚══════╝   ╚═╝   ╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
"#;

/*  */

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();

    for _ in 0..3 {
        app.on_tick();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    loop {
        // Update system info
        app.on_tick();

        /* Draw the TUI */
        terminal.draw(|frame| {
            let main_chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(frame.size());

            let top_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_chunk[0]);

            let upper_chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Min(0),
                ]) // Memory, Disk, Logo
                .split(top_chunk[0]);

            /* Generating Random Colors line by line for the Logo */

            use rand::Rng;
            let mut rng = rand::thread_rng();
            let colors = [
                Color::Red,
                Color::Green,
                Color::Blue,
                Color::Yellow,
                Color::Magenta,
                Color::Cyan,
                Color::White,
            ];

            let mut lines = Vec::new();
            for line in ASCII_ART.lines() {
                lines.push(
                    Line::from(line)
                        .style(Style::default().fg(colors[rng.gen_range(0..colors.len())])),
                );
            } /* Rendering the Memory Usage Widget */

            let total_memory = app.system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
            let used_memory = app.system.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
            let available_memory = app.system.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
            let memory_usage_percent = (used_memory / total_memory) * 100.0;

            let memory_color = if memory_usage_percent < 50.0 {
                Color::Green
            } else if memory_usage_percent < 80.0 {
                Color::Yellow
            } else {
                Color::Red
            };

            let memory_lines = vec![
                Line::from(vec![
                    Span::styled("Total: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{:.2} GB", total_memory),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Used: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{:.2} GB", used_memory),
                        Style::default().fg(memory_color),
                    ),
                    Span::styled(
                        format!(" ({:.1}%)", memory_usage_percent),
                        Style::default().fg(memory_color),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Available: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{:.2} GB", available_memory),
                        Style::default().fg(Color::Green),
                    ),
                ]),
            ];

            frame.render_widget(
                Paragraph::new(memory_lines)
                    .block(
                        Block::default()
                            .title("Memory Info")
                            .fg(Color::Yellow)
                            .borders(Borders::ALL),
                    )
                    .alignment(Alignment::Left),
                upper_chunk[0],
            );

            /* Rendering the Disk Usage Widget */

            let mut disk_lines = Vec::new();

            for disk in app.disks.iter().take(3) {
                // Show first 3 disks
                let total_space = disk.total_space();
                let available_space = disk.available_space();
                let used_space = total_space - available_space;
                let usage_percent = if total_space > 0 {
                    (used_space as f64 / total_space as f64) * 100.0
                } else {
                    0.0
                };

                let usage_color = if usage_percent > 90.0 {
                    Color::Red
                } else if usage_percent > 75.0 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                disk_lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}: ", disk.name().to_string_lossy()),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!(
                            "{:.1}GB/{:.1}GB ",
                            used_space as f64 / 1_073_741_824.0,
                            total_space as f64 / 1_073_741_824.0
                        ),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("({:.1}%)", usage_percent),
                        Style::default().fg(usage_color),
                    ),
                ]));
            }

            frame.render_widget(
                Paragraph::new(disk_lines)
                    .block(
                        Block::default()
                            .title("Disk Usage")
                            .fg(Color::Cyan)
                            .borders(Borders::ALL),
                    )
                    .alignment(Alignment::Left),
                upper_chunk[1],
            );

            frame.render_widget(
                Paragraph::new(lines)
                    .block(Block::default())
                    .alignment(Alignment::Center),
                upper_chunk[2],
            );

            frame.render_widget(
                Block::default()
                    .title("CPU Usage")
                    .borders(Borders::ALL)
                    .fg(Color::Green),
                top_chunk[1],
            );

            frame.render_widget(
                Block::default()
                    .title("Processes")
                    .borders(Borders::ALL)
                    .fg(Color::Magenta),
                main_chunk[1],
            );

            let cpus = app.system.cpus();
            let mut cpu_lines: Vec<Line> = Vec::new();

            cpu_lines.push(Line::from(Span::styled(
                "CPU Usage",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            cpu_lines.push(Line::from("")); // Add a blank line for spacing

            for (i, cpu) in cpus.iter().enumerate() {
                let usage = cpu.cpu_usage();
                let bar_color = if usage < 30.0 {
                    Color::Green
                } else if usage < 70.0 {
                    Color::Yellow
                } else {
                    Color::Red
                };

                // Create the bar itself
                let bar_width = 10; // We can set a fixed width for the bars
                let filled_bar_count = (usage / 100.0 * bar_width as f32).round() as usize;
                let empty_bar_count = bar_width - filled_bar_count;
                let bar = "█".repeat(filled_bar_count) + &" ".repeat(empty_bar_count);

                // Create a styled line with the CPU name, the bar, and the percentage
                let line = Line::from(vec![
                    Span::styled(format!("CPU {:<2}", i), Style::default().fg(Color::White)),
                    Span::raw(" ["),
                    Span::styled(bar, Style::default().fg(bar_color)),
                    Span::raw("] "),
                    Span::styled(format!("{:.2}%", usage), Style::default().fg(bar_color)),
                ]);
                cpu_lines.push(line);
            }

            frame.render_widget(
                Paragraph::new(cpu_lines)
                    .block(Block::default().borders(Borders::ALL).fg(Color::Green)),
                top_chunk[1],
            );

            let processes = app.system.processes();

            let widths = [
                Constraint::Length(10), // PID
                Constraint::Min(20),    // Name
                Constraint::Length(10), // CPU%
                Constraint::Length(12), // Memory
            ];

            // Collect and sort processes by CPU usage (highest first)
            let mut process_list: Vec<_> = processes.iter().collect();
            process_list.sort_by(|a, b| {
                b.1.cpu_usage()
                    .partial_cmp(&a.1.cpu_usage())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let rows: Vec<Row> = process_list
                .iter()
                .take(20) // Show top 20 processes by CPU usage
                .map(|(pid, process)| {
                    let cpu_usage_raw = process.cpu_usage();
                    // Cap CPU usage at 100% for display (but still sort by actual values)
                    let cpu_usage_display = cpu_usage_raw.min(100.0);
                    let cpu_usage_str = format!("{:.1}%", cpu_usage_display);
                    let memory_mb = process.memory() as f64 / 1024.0 / 1024.0;
                    let memory_str = format!("{:.1} MB", memory_mb);

                    // Color code based on displayed CPU usage (capped at 100%)
                    let cpu_color = if cpu_usage_display > 80.0 {
                        Color::Red
                    } else if cpu_usage_display > 50.0 {
                        Color::Yellow
                    } else if cpu_usage_display > 20.0 {
                        Color::Green
                    } else if cpu_usage_display > 5.0 {
                        Color::Cyan
                    } else {
                        Color::Gray
                    };

                    // Color code based on memory usage
                    let memory_color = if memory_mb > 1000.0 {
                        // > 1GB
                        Color::Red
                    } else if memory_mb > 500.0 {
                        // > 500MB
                        Color::Yellow
                    } else if memory_mb > 100.0 {
                        // > 100MB
                        Color::Green
                    } else {
                        Color::Gray
                    };

                    Row::new(vec![
                        Span::styled(pid.to_string(), Style::default().fg(Color::Blue)),
                        Span::styled(
                            process.name().to_string_lossy().to_string(),
                            Style::default().fg(Color::White),
                        ),
                        Span::styled(
                            cpu_usage_str,
                            Style::default().fg(cpu_color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(memory_str, Style::default().fg(memory_color)),
                    ])
                })
                .collect();

            let process_table = Table::new(rows, widths)
                .header(
                    Row::new(vec!["PID", "Name", "CPU% (max 100)", "Memory"]).style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Processes")
                        .fg(Color::Magenta),
                )
                .column_spacing(1)
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );

            frame.render_stateful_widget(process_table, main_chunk[1], &mut app.processes_state);
        })?;

        // Handle events with timeout - ignore all non-quit key events
        if let Ok(true) = event::poll(std::time::Duration::from_millis(16)) {
            // ~60fps
            if let Ok(evt) = event::read() {
                match evt {
                    Event::Key(key) => {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                            _ => {} // Ignore all other keys
                        }
                    }
                    // Silently ignore ALL other events (mouse, scroll, resize, etc)
                    _ => {}
                }
            }
        }

        // Control update frequency
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
