use crate::process::{ProcessManager, ProcessFilter, SortColumn, signals};
use crate::logging::log_process_operation;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{
        Block, Borders, Clear, Paragraph, Row, Table, TableState,
        Wrap, Sparkline,
    },
    Frame, Terminal,
};
use regex::Regex;
use std::io;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Process error: {0}")]
    Process(#[from] anyhow::Error),
}

pub struct App {
    process_manager: ProcessManager,
    table_state: TableState,
    sort_column: SortColumn,
    sort_ascending: bool,
    filter: ProcessFilter,
    search_mode: bool,
    search_input: String,
    show_help: bool,
    show_tree_view: bool,
    selected_process: Option<u32>,
    last_refresh: Instant,
    refresh_interval: Duration,
    status_message: Option<String>,
    show_kill_dialog: bool,
    kill_signal: i32,
    show_graphs: bool,
    cpu_history: Vec<u64>,
    memory_history: Vec<u64>,
    max_history_len: usize,
}

impl App {
    pub fn new() -> Result<Self, UiError> {
        let mut process_manager = ProcessManager::new();
        process_manager.refresh()?;

        Ok(Self {
            process_manager,
            table_state: TableState::default(),
            sort_column: SortColumn::CpuUsage,
            sort_ascending: false,
            filter: ProcessFilter::new(),
            search_mode: false,
            search_input: String::new(),
            show_help: false,
            show_tree_view: false,
            selected_process: None,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2),
            status_message: None,
            show_kill_dialog: false,
            kill_signal: signals::SIGTERM,
            show_graphs: true,
            cpu_history: Vec::new(),
            memory_history: Vec::new(),
            max_history_len: 60,
        })
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), UiError> {
        loop {
            // Refresh process data periodically
            if self.last_refresh.elapsed() >= self.refresh_interval {
                self.process_manager.refresh()?;
                self.update_history();
                self.last_refresh = Instant::now();
            }

            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_input(key.code, key.modifiers)? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_input(&mut self, key: KeyCode, _modifiers: KeyModifiers) -> Result<bool, UiError> {
        // Global quit
        if key == KeyCode::Char('q') && !self.search_mode && !self.show_kill_dialog {
            return Ok(true);
        }

        if self.show_kill_dialog {
            return Ok(self.handle_kill_dialog_input(key));
        }

        if self.search_mode {
            return Ok(self.handle_search_input(key));
        }

        match key {
            KeyCode::Char('h') | KeyCode::F(1) => {
                self.show_help = !self.show_help;
            }
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_input.clear();
            }
            KeyCode::Char('t') => {
                self.show_tree_view = !self.show_tree_view;
            }
            KeyCode::Char('g') => {
                self.show_graphs = !self.show_graphs;
                self.status_message = Some(format!(
                    "Graphs: {}",
                    if self.show_graphs { "ON" } else { "OFF" }
                ));
            }
            KeyCode::Char('k') => {
                if let Some(selected) = self.get_selected_process_id() {
                    self.selected_process = Some(selected);
                    self.show_kill_dialog = true;
                    self.kill_signal = signals::SIGTERM;
                }
            }
            KeyCode::Up => {
                self.previous_process();
            }
            KeyCode::Down => {
                self.next_process();
            }
            KeyCode::Char('r') => {
                self.process_manager.refresh()?;
                self.status_message = Some("Refreshed process list".to_string());
            }
            KeyCode::F(5) => {
                self.process_manager.refresh()?;
                self.status_message = Some("Refreshed process list".to_string());
            }
            // Sorting
            KeyCode::Char('p') => {
                self.set_sort_column(SortColumn::Pid);
            }
            KeyCode::Char('n') => {
                self.set_sort_column(SortColumn::Name);
            }
            KeyCode::Char('u') => {
                self.set_sort_column(SortColumn::User);
            }
            KeyCode::Char('c') => {
                self.set_sort_column(SortColumn::CpuUsage);
            }
            KeyCode::Char('m') => {
                self.set_sort_column(SortColumn::MemoryUsage);
            }
            KeyCode::Char('s') => {
                self.set_sort_column(SortColumn::StartTime);
            }
            // Filter toggles
            KeyCode::Char('o') => {
                self.filter.show_only_user_processes = !self.filter.show_only_user_processes;
                self.status_message = Some(format!(
                    "User processes only: {}",
                    if self.filter.show_only_user_processes { "ON" } else { "OFF" }
                ));
            }
            _ => {}
        }

        Ok(false)
    }

    fn handle_search_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Enter => {
                self.search_mode = false;
                if !self.search_input.is_empty() {
                    match Regex::new(&self.search_input) {
                        Ok(regex) => {
                            self.filter.name_pattern = Some(regex);
                            self.status_message = Some(format!("Search: {}", self.search_input));
                        }
                        Err(_) => {
                            self.status_message = Some("Invalid regex pattern".to_string());
                        }
                    }
                } else {
                    self.filter.name_pattern = None;
                    self.status_message = Some("Search cleared".to_string());
                }
            }
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_input.clear();
            }
            KeyCode::Backspace => {
                self.search_input.pop();
            }
            KeyCode::Char(c) => {
                self.search_input.push(c);
            }
            _ => {}
        }
        false
    }

    fn handle_kill_dialog_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Enter => {
                if let Some(pid) = self.selected_process {
                    // Get process info for logging before killing it
                    let process_name = self.process_manager.get_process(pid)
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| format!("{}", pid));
                    let user = users::get_current_username()
                        .and_then(|os_str| os_str.into_string().ok())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    match self.process_manager.kill_process(pid, self.kill_signal) {
                        Ok(()) => {
                            self.status_message = Some(format!("Sent signal {} to process {}", self.kill_signal, pid));
                            // Log successful operation
                            log_process_operation(
                                "kill",
                                pid,
                                &process_name,
                                &user,
                                true,
                                Some(&format!("signal: {}", self.kill_signal))
                            );
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Failed to kill process: {}", e));
                            // Log failed operation
                            log_process_operation(
                                "kill",
                                pid,
                                &process_name,
                                &user,
                                false,
                                Some(&format!("error: {}", e))
                            );
                        }
                    }
                }
                self.show_kill_dialog = false;
                self.selected_process = None;
            }
            KeyCode::Esc => {
                self.show_kill_dialog = false;
                self.selected_process = None;
            }
            KeyCode::Char('1') => self.kill_signal = signals::SIGHUP,
            KeyCode::Char('2') => self.kill_signal = signals::SIGINT,
            KeyCode::Char('3') => self.kill_signal = signals::SIGQUIT,
            KeyCode::Char('9') => self.kill_signal = signals::SIGKILL,
            KeyCode::Char('t') => self.kill_signal = signals::SIGTERM,
            KeyCode::Char('s') => self.kill_signal = signals::SIGSTOP,
            KeyCode::Char('c') => self.kill_signal = signals::SIGCONT,
            _ => {}
        }
        false
    }

    fn set_sort_column(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = false;
        }
    }

    fn get_selected_process_id(&self) -> Option<u32> {
        if let Some(selected) = self.table_state.selected() {
            let processes = if self.show_tree_view {
                // For tree view, we'd need to implement tree structure
                self.process_manager.sort_processes(self.sort_column.clone(), self.sort_ascending)
            } else {
                self.process_manager.sort_processes(self.sort_column.clone(), self.sort_ascending)
            };
            
            let filtered_processes: Vec<_> = processes.iter()
                .filter(|p| self.filter.matches(p))
                .collect();
            
            filtered_processes.get(selected).map(|p| p.pid)
        } else {
            None
        }
    }

    fn next_process(&mut self) {
        let len = self.get_filtered_process_count();
        if len > 0 {
            let selected = self.table_state.selected().unwrap_or(0);
            self.table_state.select(Some((selected + 1) % len));
        }
    }

    fn previous_process(&mut self) {
        let len = self.get_filtered_process_count();
        if len > 0 {
            let selected = self.table_state.selected().unwrap_or(0);
            self.table_state.select(Some(if selected == 0 { len - 1 } else { selected - 1 }));
        }
    }

    fn get_filtered_process_count(&self) -> usize {
        let processes = self.process_manager.sort_processes(self.sort_column.clone(), self.sort_ascending);
        processes.iter().filter(|p| self.filter.matches(p)).count()
    }

    fn update_history(&mut self) {
        let system_info = self.process_manager.get_system_info();
        
        // Calculate overall CPU usage (sum of all processes)
        let processes = self.process_manager.get_processes();
        let total_cpu: f32 = processes.iter().map(|p| p.cpu_usage).sum();
        
        // Store as percentage scaled to 0-100
        self.cpu_history.push(total_cpu.min(100.0) as u64);
        if self.cpu_history.len() > self.max_history_len {
            self.cpu_history.remove(0);
        }
        
        // Memory usage percentage
        let memory_percent = if system_info.total_memory > 0 {
            ((system_info.used_memory as f64 / system_info.total_memory as f64) * 100.0) as u64
        } else {
            0
        };
        self.memory_history.push(memory_percent);
        if self.memory_history.len() > self.max_history_len {
            self.memory_history.remove(0);
        }
    }

    fn render_system_graphs(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);
        
        // CPU Graph
        let cpu_data: Vec<u64> = self.cpu_history.clone();
        let cpu_max = cpu_data.iter().max().copied().unwrap_or(100).max(1);
        
        let cpu_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("CPU Usage History (Max: {}%)", cpu_max))
            )
            .data(&cpu_data)
            .style(Style::default().fg(Color::Cyan))
            .max(cpu_max);
        
        f.render_widget(cpu_sparkline, chunks[0]);
        
        // Memory Graph  
        let mem_data: Vec<u64> = self.memory_history.clone();
        let mem_max = mem_data.iter().max().copied().unwrap_or(100).max(1);
        
        let mem_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Memory Usage History (Max: {}%)", mem_max))
            )
            .data(&mem_data)
            .style(Style::default().fg(Color::Green))
            .max(mem_max);
        
        f.render_widget(mem_sparkline, chunks[1]);
    }

    fn ui(&mut self, f: &mut Frame) {
        let main_chunks = if self.show_graphs {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // System info
                    Constraint::Length(10), // Graphs
                    Constraint::Min(0),     // Process table
                    Constraint::Length(3),  // Status bar
                ])
                .split(f.size())
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // System info
                    Constraint::Min(0),    // Process table
                    Constraint::Length(3), // Status bar
                ])
                .split(f.size())
        };

        // System info
        self.render_system_info(f, main_chunks[0]);

        // Graphs (if enabled)
        if self.show_graphs {
            self.render_system_graphs(f, main_chunks[1]);
            // Process table
            self.render_process_table(f, main_chunks[2]);
            // Status bar
            self.render_status_bar(f, main_chunks[3]);
        } else {
            // Process table
            self.render_process_table(f, main_chunks[1]);
            // Status bar
            self.render_status_bar(f, main_chunks[2]);
        }

        // Overlays
        if self.show_help {
            self.render_help_popup(f);
        }

        if self.search_mode {
            self.render_search_popup(f);
        }

        if self.show_kill_dialog {
            self.render_kill_dialog(f);
        }
    }

    fn render_system_info(&self, f: &mut Frame, area: Rect) {
        let system_info = self.process_manager.get_system_info();
        
        let memory_percent = (system_info.used_memory as f64 / system_info.total_memory as f64) * 100.0;
        let swap_percent = if system_info.total_swap > 0 {
            (system_info.used_swap as f64 / system_info.total_swap as f64) * 100.0
        } else {
            0.0
        };

        let info_text = format!(
            "CPUs: {} | Load: {:.2} {:.2} {:.2} | Memory: {:.1}% ({} MB / {} MB) | Swap: {:.1}% | Uptime: {}d {}h {}m",
            system_info.cpu_count,
            system_info.load_average.one,
            system_info.load_average.five,
            system_info.load_average.fifteen,
            memory_percent,
            system_info.used_memory / 1024,
            system_info.total_memory / 1024,
            swap_percent,
            system_info.uptime / 86400,
            (system_info.uptime % 86400) / 3600,
            (system_info.uptime % 3600) / 60,
        );

        let paragraph = Paragraph::new(info_text)
            .block(Block::default().borders(Borders::ALL).title("System Information"))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_process_table(&mut self, f: &mut Frame, area: Rect) {
        let processes = self.process_manager.sort_processes(self.sort_column.clone(), self.sort_ascending);
        let filtered_processes: Vec<_> = processes.iter()
            .filter(|p| self.filter.matches(p))
            .collect();

        let header = Row::new(vec![
            "PID", "User", "CPU%", "Mem%", "Memory", "Net", "Status", "Name", "Command"
        ])
        .style(Style::default().fg(Color::Yellow))
        .height(1);

        let rows: Vec<Row> = filtered_processes.iter().map(|process| {
            // Format network connections
            let net_info = if let Some(connections) = process.network_connections {
                format!("{}", connections)
            } else {
                "-".to_string()
            };
            
            // Format status with container/GPU indicators
            let mut status_str = process.status.clone();
            if process.is_container {
                status_str.push_str(" 🐳");
            }
            if process.gpu_memory.is_some() {
                status_str.push_str(" 🎮");
            }
            
            Row::new(vec![
                process.pid.to_string(),
                process.user.clone(),
                format!("{:.1}", process.cpu_usage),
                format!("{:.1}", process.memory_percent),
                format!("{} KB", process.memory_usage),
                net_info,
                status_str,
                process.name.clone(),
                if process.command.len() > 40 {
                    format!("{}...", &process.command[..37])
                } else {
                    process.command.clone()
                },
            ])
        }).collect();

        let sort_indicator = if self.sort_ascending { "▲" } else { "▼" };
        let title = format!(
            "Processes ({}) - Sort: {:?} {} - Press 'h' for help",
            filtered_processes.len(),
            self.sort_column,
            sort_indicator
        );

        let table = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .widths(&[
                Constraint::Length(8),  // PID
                Constraint::Length(10), // User
                Constraint::Length(6),  // CPU%
                Constraint::Length(6),  // Mem%
                Constraint::Length(10), // Memory
                Constraint::Length(5),  // Net
                Constraint::Length(8),  // State
                Constraint::Length(12), // Name
                Constraint::Min(20),    // Command
            ]);

        f.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_text = if let Some(ref message) = self.status_message {
            format!("Status: {} | Press 'q' to quit, 'h' for help", message)
        } else {
            "Press 'q' to quit, 'h' for help, '/' to search, 'k' to kill process".to_string()
        };

        let paragraph = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        f.render_widget(paragraph, area);
    }

    fn render_help_popup(&self, f: &mut Frame) {
        let area = centered_rect(60, 70, f.size());
        
        let help_text = vec![
            Line::from("Linux Process Manager - Help"),
            Line::from(""),
            Line::from("Navigation:"),
            Line::from("  ↑/↓        Navigate process list"),
            Line::from("  q          Quit application"),
            Line::from("  r/F5       Refresh process list"),
            Line::from(""),
            Line::from("Sorting (click column or use key):"),
            Line::from("  p          Sort by PID"),
            Line::from("  n          Sort by Name"),
            Line::from("  u          Sort by User"),
            Line::from("  c          Sort by CPU usage"),
            Line::from("  m          Sort by Memory usage"),
            Line::from("  s          Sort by Start time"),
            Line::from(""),
            Line::from("Actions:"),
            Line::from("  k          Kill selected process"),
            Line::from("  /          Search processes"),
            Line::from("  t          Toggle tree view"),
            Line::from("  g          Toggle system graphs"),
            Line::from("  o          Toggle user processes only"),
            Line::from(""),
            Line::from("Kill Dialog Signals:"),
            Line::from("  t          SIGTERM (15) - Graceful termination"),
            Line::from("  9          SIGKILL (9) - Force kill"),
            Line::from("  1          SIGHUP (1) - Hang up"),
            Line::from("  2          SIGINT (2) - Interrupt"),
            Line::from("  s          SIGSTOP (19) - Stop process"),
            Line::from("  c          SIGCONT (18) - Continue process"),
            Line::from(""),
            Line::from("Press 'h' or F1 to close this help"),
        ];

        let paragraph = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    fn render_search_popup(&self, f: &mut Frame) {
        let area = centered_rect(50, 20, f.size());
        
        let text = format!("Search: {}", self.search_input);
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Search Processes (regex)"));

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    fn render_kill_dialog(&self, f: &mut Frame) {
        let area = centered_rect(40, 30, f.size());
        
        let process_info = if let Some(pid) = self.selected_process {
            if let Some(process) = self.process_manager.get_process(pid) {
                format!("PID: {}\nName: {}\nUser: {}", process.pid, process.name, process.user)
            } else {
                format!("PID: {}", pid)
            }
        } else {
            "No process selected".to_string()
        };

        let signal_name = match self.kill_signal {
            signals::SIGTERM => "SIGTERM (graceful)",
            signals::SIGKILL => "SIGKILL (force)",
            signals::SIGHUP => "SIGHUP",
            signals::SIGINT => "SIGINT",
            signals::SIGQUIT => "SIGQUIT", 
            signals::SIGSTOP => "SIGSTOP",
            signals::SIGCONT => "SIGCONT",
            _ => "Unknown",
        };

        let text = format!(
            "{}\n\nSignal: {} ({})\n\nPress Enter to confirm, Esc to cancel\nKeys: t=TERM, 9=KILL, 1=HUP, 2=INT, s=STOP, c=CONT",
            process_info, self.kill_signal, signal_name
        );

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Kill Process"))
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn run_app() -> Result<(), UiError> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new()?;
    let res = app.run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}