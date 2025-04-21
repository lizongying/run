mod conf;
mod error;
mod shell;

use crossterm::{
    ExecutableCommand,
    event::{self, KeyCode, KeyEvent},
    terminal,
};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Direction, Layout},
    widgets::{Block, Borders, List, ListItem},
};

use crate::shell::Shell;
use crossterm::terminal::ClearType;
use std::io;
use tui::style::{Color, Style};
use tui::widgets::Paragraph;

use crate::conf::{Conf, load_config};

#[derive(Debug)]
enum TaskType {
    Direct,
    WithOptions,
}

struct Job<'a> {
    job: &'a conf::Job,
    task_type: TaskType,
    action: Box<dyn Fn(&mut Shell, &conf::Job, Option<&conf::OptionItem>) -> String + Send + Sync>,
}

impl Debug for Job<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Job")
            .field("label", &self.job.label)
            .field("options", &self.job.options)
            .field("default_option", &self.job.default_option)
            .field("task_type", &self.task_type)
            .field("action", &"Fn action")
            .finish()
    }
}

fn run_shell_command(shell: &mut Shell, cmd: &str) -> String {
    shell
        .execute(cmd.to_string())
        .unwrap_or_else(|_| String::new())
}

fn create_jobs(conf: &Conf) -> Vec<Job> {
    conf.jobs
        .iter()
        .map(|job| {
            let mut task_type = TaskType::Direct;
            let action: Box<
                dyn Fn(&mut Shell, &conf::Job, Option<&conf::OptionItem>) -> String + Send + Sync,
            > = match &job.options {
                Some(_) => {
                    task_type = TaskType::WithOptions;

                    Box::new(move |shell, job, option| {
                        if let Some(selected_option) = option {
                            format!(
                                "{} {}:\n{}",
                                &job.label,
                                selected_option.label,
                                run_shell_command(shell, &selected_option.cmd)
                            )
                        } else {
                            String::new()
                        }
                    })
                }
                None => Box::new(move |shell, job, _| {
                    if let Some(cmd) = &job.cmd {
                        format!("{}:\n{}", &job.label, run_shell_command(shell, cmd))
                    } else {
                        match job.label.as_str() {
                            "Info" => format!("Which Shell:\n{}", shell.get_shell()),
                            "Exit" => std::process::exit(0),
                            _ => String::new(),
                        }
                    }
                }),
            };

            Job {
                job,
                task_type,
                action,
            }
        })
        .collect()
}

fn main() -> Result {
    if let Err(e) = terminal::enable_raw_mode() {
        eprintln!("Failed to enable raw mode: {}", e);
        std::process::exit(1);
    }

    // Clear terminal screen at the beginning
    if let Err(e) = io::stdout().execute(terminal::Clear(ClearType::All)) {
        eprintln!("Failed to clear terminal: {}", e);
        std::process::exit(1);
    }

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap_or_else(|e| {
        eprintln!("Failed to create terminal: {}", e);
        std::process::exit(1);
    });

    let conf = load_config().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {}", e);
        std::process::exit(1);
    });

    let jobs = create_jobs(&conf);

    let mut option_indices: HashMap<usize, usize> = HashMap::new();
    let mut selected_index = 0;
    let mut output_message = String::new();
    let mut selecting_option = false;
    let mut selected_job: Option<&Job> = None;

    let mut shell = Shell::new().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    loop {
        terminal
            .draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(2)
                    .constraints(
                        [
                            tui::layout::Constraint::Percentage(30),
                            tui::layout::Constraint::Percentage(30),
                            tui::layout::Constraint::Percentage(40),
                        ]
                        .as_ref(),
                    )
                    .split(size);

                let items: Vec<ListItem> = jobs
                    .iter()
                    .enumerate()
                    .map(|(i, job)| {
                        let style = if i == selected_index {
                            Style::default()
                                .bg(if selecting_option {
                                    Color::Gray
                                } else {
                                    Color::Blue
                                })
                                .fg(Color::White)
                        } else {
                            Style::default()
                        };
                        ListItem::new(job.job.label.as_str()).style(style)
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Jobs"))
                    .highlight_style(Style::default().bg(Color::Blue));

                f.render_widget(list, chunks[0]);

                if let Some(ref options) = jobs[selected_index].job.options {
                    let items: Vec<ListItem> = options
                        .iter()
                        .enumerate()
                        .map(|(i, option)| {
                            let option_index = if option_indices.contains_key(&selected_index) {
                                option_indices[&selected_index]
                            } else {
                                let index = jobs[selected_index].job.default_option.unwrap_or(0);
                                option_indices.insert(selected_index, index);
                                index
                            };

                            let style = if i == option_index {
                                Style::default()
                                    .bg(if selecting_option {
                                        Color::Blue
                                    } else {
                                        Color::Gray
                                    })
                                    .fg(Color::White)
                            } else {
                                Style::default()
                            };

                            ListItem::new(option.label.as_str()).style(style)
                        })
                        .collect();

                    let select_list = List::new(items)
                        .block(Block::default().borders(Borders::ALL).title("Options"))
                        .highlight_style(Style::default().bg(Color::Blue));

                    f.render_widget(select_list, chunks[1]);
                } else {
                    let empty_list = List::new(Vec::new())
                        .block(Block::default().borders(Borders::ALL).title("Option"))
                        .highlight_style(Style::default().bg(Color::Blue));
                    f.render_widget(empty_list, chunks[1]);
                }

                let output_paragraph = Paragraph::new(output_message.as_str())
                    .block(Block::default().borders(Borders::ALL).title("Output"));
                f.render_widget(output_paragraph, chunks[2]);
            })
            .expect("Failed to create terminal");

        match event::poll(std::time::Duration::from_millis(500)) {
            Ok(true) => match event::read() {
                Ok(event::Event::Key(KeyEvent {
                    code,
                    modifiers: _modifiers,
                    ..
                })) => match code {
                    KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Down => {
                        if !selecting_option {
                            selected_index = (selected_index + 1) % jobs.len();
                        } else {
                            if let Some(job) = selected_job.as_ref() {
                                if let Some(ref options) = job.job.options {
                                    let current_index = *option_indices
                                        .get(&selected_index)
                                        .unwrap_or(&job.job.default_option.unwrap_or(0));

                                    let new_index = (current_index + 1) % options.len();

                                    option_indices.insert(selected_index, new_index);
                                }
                            }
                        }
                    }
                    KeyCode::Up => {
                        if !selecting_option {
                            selected_index = if selected_index == 0 {
                                jobs.len() - 1
                            } else {
                                selected_index - 1
                            };
                        } else {
                            if let Some(job) = selected_job.as_ref() {
                                if let Some(ref options) = job.job.options {
                                    let current_index = *option_indices
                                        .get(&selected_index)
                                        .unwrap_or(&job.job.default_option.unwrap_or(0));

                                    let new_index = if current_index == 0 {
                                        options.len() - 1
                                    } else {
                                        current_index - 1
                                    };

                                    option_indices.insert(selected_index, new_index);
                                }
                            }
                        }
                    }
                    KeyCode::Left => {
                        if selecting_option {
                            selecting_option = false;
                        }
                    }
                    KeyCode::Right => {
                        if !selecting_option {
                            selected_job = Some(&jobs[selected_index]);

                            match selected_job.unwrap().task_type {
                                TaskType::Direct => {}
                                TaskType::WithOptions => {
                                    selecting_option = true;
                                }
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if !selecting_option {
                            selected_job = Some(&jobs[selected_index]);
                            if let Some(job) = selected_job {
                                match job.task_type {
                                    TaskType::Direct => {
                                        output_message = (job.action)(&mut shell, job.job, None);
                                    }
                                    TaskType::WithOptions => {
                                        selecting_option = true;
                                    }
                                }
                            }
                        } else {
                            if let Some(job) = selected_job {
                                if let Some(ref options) = job.job.options {
                                    let current_index = *option_indices
                                        .get(&selected_index)
                                        .unwrap_or(&job.job.default_option.unwrap_or(0));

                                    let option = &options[current_index];

                                    let result = (job.action)(&mut shell, job.job, Some(option));
                                    output_message = result;
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Err(e) => {
                    eprintln!("Error reading event: {}", e);
                }
                _ => {}
            },
            Ok(false) => {
                continue;
            }
            Err(e) => {
                eprintln!("Error polling event: {}", e);
                break;
            }
        }
    }

    if let Err(e) = terminal::disable_raw_mode() {
        eprintln!("Error disabling raw mode: {}", e);
        std::process::exit(1);
    }
    Ok(())
}
