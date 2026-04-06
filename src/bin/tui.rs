//! Real-time BrainBit EEG terminal UI using ratatui.
//!
//! Run with: `cargo run --bin brainbit-tui`

use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use brainbit::prelude::*;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::*,
};

const DISPLAY_SAMPLES: usize = 500; // ~2 seconds at 250 Hz

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // ── Scan ─────────────────────────────────────────────────────────────
    println!("Scanning for BrainBit devices (5 seconds)...");
    let families = [
        SensorFamily::LEBrainBit,
        SensorFamily::LEBrainBit2,
        SensorFamily::LEBrainBitPro,
        SensorFamily::LEBrainBitFlex,
    ];
    let scanner = Scanner::new(&families)?;
    scanner.start()?;
    std::thread::sleep(Duration::from_secs(5));
    scanner.stop()?;

    let devices = scanner.devices()?;
    if devices.is_empty() {
        eprintln!("No BrainBit device found.");
        return Ok(());
    }
    println!("Connecting to {}...", devices[0].name_str());
    let mut device = BrainBitDevice::connect(&scanner, &devices[0])?;

    let device_name = device.name().unwrap_or_else(|_| "Unknown".into());
    let battery = device.battery_level().unwrap_or(-1);
    let freq = device.sampling_frequency().ok();

    // ── Shared ring buffer ───────────────────────────────────────────────
    let ring = Arc::new(Mutex::new(RingBuffer::new(DISPLAY_SAMPLES)));
    let ring2 = ring.clone();

    device.on_signal(move |samples| {
        let mut buf = ring2.lock().unwrap();
        for s in samples {
            buf.push(*s);
        }
    })?;
    device.start_signal()?;

    // ── Terminal setup ───────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(33); // ~30 fps
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            let buf = ring.lock().unwrap();
            let area = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(4),
                    Constraint::Min(4),
                    Constraint::Min(4),
                    Constraint::Min(4),
                ])
                .split(area);

            // Title bar
            let title = format!(
                " BrainBit EEG — {} | Battery: {}% | Freq: {:?} | Samples: {} | q to quit ",
                device_name, battery, freq, buf.len,
            );
            let title_block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(title_block, chunks[0]);

            // Render each channel
            let channel_names = BRAINBIT_CHANNEL_NAMES;
            let colors = [Color::Green, Color::Yellow, Color::Blue, Color::Magenta];

            for (ch, (&name, color)) in channel_names.iter().zip(colors.iter()).enumerate() {
                let data: Vec<(f64, f64)> = buf
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (i as f64, s.channels[ch] * 1e6)) // convert V → µV
                    .collect();

                let dataset = Dataset::default()
                    .name(name)
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(*color))
                    .data(&data);

                let y_min = data.iter().map(|d| d.1).fold(f64::INFINITY, f64::min);
                let y_max = data.iter().map(|d| d.1).fold(f64::NEG_INFINITY, f64::max);
                let margin = (y_max - y_min).max(1.0) * 0.1;

                let chart = Chart::new(vec![dataset])
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!(" {} (µV) ", name)),
                    )
                    .x_axis(
                        Axis::default()
                            .bounds([0.0, DISPLAY_SAMPLES as f64]),
                    )
                    .y_axis(
                        Axis::default()
                            .bounds([y_min - margin, y_max + margin])
                            .labels::<Vec<Line>>(vec![
                                format!("{:.0}", y_min - margin).into(),
                                format!("{:.0}", y_max + margin).into(),
                            ]),
                    );

                f.render_widget(chart, chunks[1 + ch]);
            }
        })?;

        // Event handling
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // ── Cleanup ──────────────────────────────────────────────────────────
    device.stop_signal()?;
    device.remove_signal_callback();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!("Bye!");
    Ok(())
}

/// Fixed-size ring buffer for EEG samples.
struct RingBuffer {
    data: Vec<EegSample>,
    capacity: usize,
    len: usize,
    write_pos: usize,
}

impl RingBuffer {
    fn new(capacity: usize) -> Self {
        RingBuffer {
            data: Vec::with_capacity(capacity),
            capacity,
            len: 0,
            write_pos: 0,
        }
    }

    fn push(&mut self, sample: EegSample) {
        if self.data.len() < self.capacity {
            self.data.push(sample);
        } else {
            self.data[self.write_pos] = sample;
        }
        self.write_pos = (self.write_pos + 1) % self.capacity;
        self.len = (self.len + 1).min(usize::MAX);
    }

    /// Iterate samples in chronological order.
    fn iter(&self) -> Box<dyn Iterator<Item = &EegSample> + '_> {
        if self.data.len() < self.capacity {
            Box::new(self.data.iter())
        } else {
            Box::new(
                self.data[self.write_pos..]
                    .iter()
                    .chain(self.data[..self.write_pos].iter()),
            )
        }
    }
}
