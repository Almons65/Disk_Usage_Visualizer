use iced::{
    Application, Command, Element, Length, Settings, Subscription,
    widget::{Button, Column, Container, ProgressBar, Text, TextInput, Row, Space},
};
use sysinfo::{System, SystemExt, DiskExt};
use rayon::prelude::*;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;
use std::fs::{self, File};
use std::time::{Instant, Duration};
use serde::{Serialize, Deserialize};
use serde_json;
use csv::Writer;

pub fn main() -> iced::Result {
    DiskVisualizer::run(Settings::default())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiskInfo {
    name: String,
    total_space: f64,
    used_space: f64,
    files: Vec<FileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileInfo {
    path: String,
    size_mb: f64, 
}

struct DiskVisualizer {
    disks: Vec<DiskInfo>,
    scanning: bool,
    error_message: Option<String>,
    scan_duration: Option<f64>,
    scan_count: Arc<AtomicUsize>,
    file_type_filter: String,
    file_name_filter: String,
    elapsed_time: Duration,
}

#[derive(Debug, Clone)]
pub enum Message {
    Scan,
    StopScan,
    Scanned(Result<(Vec<DiskInfo>, f64), String>),
    Refresh,
    FileTypeFilterChanged(String),
    FileNameFilterChanged(String),
    ExportAsJson,
    ExportAsCsv,
    ExportCompleted(Result<(), String>),
    Done,
    Tick,
}

impl Application for DiskVisualizer {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = iced::theme::Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            DiskVisualizer {
                disks: Vec::new(),
                scanning: false,
                error_message: None,
                scan_duration: None,
                scan_count: Arc::new(AtomicUsize::new(0)),
                file_type_filter: String::new(),
                file_name_filter: String::new(),
                elapsed_time: Duration::from_secs(0),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Disk Usage Visualizer")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Scan => {
                self.scanning = true;
                self.elapsed_time = Duration::from_secs(0);
                self.error_message = None;
                self.scan_duration = None;

                let scan_count_clone = Arc::clone(&self.scan_count);
                let (tx, rx) = std::sync::mpsc::channel();

                thread::spawn(move || {
                    let start_time = Instant::now();
                    let system = System::new_all();
                    let mut disks: Vec<DiskInfo> = Vec::new();

                    for disk in system.disks() {
                        let total_space = disk.total_space() as f64 / 1_073_741_824.0;
                        let used_space = (disk.total_space() - disk.available_space()) as f64 / 1_073_741_824.0;

                        let files = Arc::new(Mutex::new(Vec::new()));

                        if total_space > 0.0 {
                            WalkDir::new(disk.mount_point())
                                .into_iter()
                                .par_bridge()
                                .filter_map(|e| e.ok())
                                .for_each(|entry| {
                                    let path = entry.path();
                                    if let Ok(metadata) = fs::metadata(path) {
                                        if metadata.is_file() {
                                            let file_info = FileInfo {
                                                path: path.display().to_string(),
                                                size_mb: metadata.len() as f64 / 1_048_576.0,
                                            };
                                            files.lock().unwrap().push(file_info);
                                        }
                                    }
                                });

                            let mut files = Arc::try_unwrap(files).unwrap().into_inner().unwrap();
                            files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());

                            disks.push(DiskInfo {
                                name: disk.name().to_string_lossy().to_string(),
                                total_space,
                                used_space,
                                files,
                            });
                        }
                    }

                    let duration = start_time.elapsed().as_secs_f64();
                    scan_count_clone.fetch_add(1, Ordering::SeqCst);

                    if disks.is_empty() {
                        let _ = tx.send(Err("Failed to retrieve disk information".to_string()));
                    } else {
                        let _ = tx.send(Ok((disks, duration)));
                    }
                });

                return Command::perform(async move {
                    let result = rx.recv().unwrap();
                    result
                }, |result: Result<(Vec<DiskInfo>, f64), String>| Message::Scanned(result));
            }
            Message::StopScan => {
                self.scanning = false;
                Command::none()
            }
            Message::Scanned(result) => {
                self.scanning = false;
                match result {
                    Ok((disks, duration)) => {
                        self.disks = disks;
                        self.scan_duration = Some(duration);
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
                Command::none()
            }
            Message::Tick => {
                if self.scanning {
                    self.elapsed_time += Duration::from_secs(1);
                }
                Command::none()
            }
            Message::ExportAsJson => {
                let disks = self.disks.clone();
                Command::perform(async move { export_to_json(disks) }, Message::ExportCompleted)
            }
            Message::ExportAsCsv => {
                let disks = self.disks.clone();
                Command::perform(async move { export_to_csv(disks) }, Message::ExportCompleted)
            }
            Message::ExportCompleted(result) => {
                self.error_message = result.err();
                Command::none()
            }
            Message::Done => {
                std::process::exit(0);
            }
            Message::Refresh => {
                self.scan_duration = None;
                Command::perform(async { Ok(()) }, |_: Result<(), ()>| Message::Scan)
            }
            Message::FileTypeFilterChanged(new_filter) => {
                self.file_type_filter = new_filter;
                Command::none()
            }
            Message::FileNameFilterChanged(new_filter) => {
                self.file_name_filter = new_filter;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
    let mut content = Column::new()
        .spacing(10)
        .padding(10)
        .max_width(800);

    
    if self.scanning {
        content = content.push(Text::new("Scanning... Please wait..."));
    } else {
        // Show error message if any
        if let Some(ref error_message) = self.error_message {
            content = content.push(Text::new(error_message).style(iced::Color::from_rgb(1.0, 0.0, 0.0)));
        }

        // File filters
        content = content.push(
            TextInput::new("File type filter (e.g., .txt, .jpg)", &self.file_type_filter)
                .on_input(Message::FileTypeFilterChanged)
                .padding(5),
        );

        content = content.push(
            TextInput::new("File name filter (e.g., report)", &self.file_name_filter)
                .on_input(Message::FileNameFilterChanged)
                .padding(5),
        );

        
        for disk in &self.disks {
            let usage_percentage = (disk.used_space / disk.total_space) * 100.0;
            content = content
                .push(Text::new(format!("Disk: {}", disk.name)))
                .push(Text::new(format!("Total Space: {:.2} GB", disk.total_space)))
                .push(Text::new(format!("Used Space: {:.2} GB", disk.used_space)))
                .push(ProgressBar::new(0.0..=100.0, usage_percentage as f32).height(10));

            
            let mut matching_files: Vec<FileInfo> = disk
                .files
                .iter()
                .filter(|file| {
                    (self.file_type_filter.is_empty() || file.path.ends_with(&self.file_type_filter)) &&
                    (self.file_name_filter.is_empty() || file.path.contains(&self.file_name_filter))
                })
                .cloned()
                .collect();

            
            matching_files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap_or(std::cmp::Ordering::Equal));
            let top_files = matching_files.iter().take(5);

            
            for file in top_files {
                let (size, unit) = if file.size_mb >= 1000.0 {
                    (file.size_mb / 1024.0, "GB")
                } else {
                    (file.size_mb, "MB")
                };
                content = content.push(Text::new(format!("File: {}, Size: {:.2} {}", file.path, size, unit)));
            }
        }
    }

    
    if self.scanning {
        content = content.push(Text::new(format!(
            "Time Elapsed: {:.0} seconds",
            self.elapsed_time.as_secs()
        )));
    } else if let Some(duration) = self.scan_duration {
        content = content.push(Text::new(format!("Scan Duration: {:.2} seconds", duration)));
    }

    content = content.push(Text::new(format!("Scans performed: {}", self.scan_count.load(Ordering::SeqCst))));

        
    content = content
        .push(Container::new(
            Button::new(Text::new("Scan Disk"))
                .on_press(Message::Scan)
                .width(Length::Fixed(80.0)),
        ))
        .push(Container::new(
            Button::new(Text::new("Stop Scan"))
                .on_press(Message::StopScan)
                .width(Length::Fixed(85.0)),
        ));

    content = content.push(
        Container::new(
            Button::new(Text::new("Refresh Disk Info"))
                .on_press(Message::Refresh)
                .width(Length::Fixed(130.0)),
        )
    );

    content = content.push(Row::new()
        .spacing(10)
        .push(Button::new(Text::new("Export as JSON")).on_press(Message::ExportAsJson).width(Length::Fixed(120.0)))
        .push(Button::new(Text::new("Export as CSV")).on_press(Message::ExportAsCsv).width(Length::Fixed(110.0)))
    );

    
    content = content.push(Space::with_height(Length::Fill));

    let scrollable_content = iced::widget::scrollable::Scrollable::new(content)
        .height(Length::Fill)
        .width(Length::Fill);

    let final_layout = Column::new()
        .spacing(10)
        .push(scrollable_content) 
        .push(Button::new(Text::new("Done")).on_press(Message::Done).width(Length::Shrink)); // "Done" button at the bottom

    
    Container::new(final_layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}
    

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.scanning {
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }
}

fn export_to_json(disks: Vec<DiskInfo>) -> Result<(), String> {
    serde_json::to_writer_pretty(&File::create("disk_usage.json").map_err(|e| e.to_string())?, &disks)
        .map_err(|e| e.to_string())
}

fn export_to_csv(disks: Vec<DiskInfo>) -> Result<(), String> {
    let mut wtr = Writer::from_writer(File::create("disk_usage.csv").map_err(|e| e.to_string())?);
    for disk in disks {
        for file in disk.files {
            wtr.write_record(&[
                &disk.name,
                &format!("{:.2}", disk.total_space),
                &format!("{:.2}", disk.used_space),
                &file.path,
                &format!("{:.2}", if file.size_mb >= 1000.0 { file.size_mb / 1024.0 } else { file.size_mb }),
                &(if file.size_mb >= 1000.0 { "GB" } else { "MB" }).to_string(), 
            ]).map_err(|e| e.to_string())?;
        }
    }
    wtr.flush().map_err(|e| e.to_string())
}
