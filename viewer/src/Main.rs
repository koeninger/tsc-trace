use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::Duration;
use std::thread;

#[derive(Clone, Copy, Debug)]
pub struct Span {
    tag: u64,
    start: u64,
    stop: u64,
}

pub struct App {
    window_width: u32,
    window_height: u32,
    background_color: Color,
    colors: Vec<Color>,
    muted_colors: Vec<Color>,
    sdl_context: sdl2::Sdl,
    canvas: WindowCanvas,
    /// span cycles per horizontal pixel
    scale: u64,
    /// vertical pixels per span
    span_height: i32,
    /// vertical pixels between spans
    span_spacing: i32,
}

impl App {
    pub fn new() -> Result<(Vec<Span>, App), String> {
        let mut spans = vec![
            Span { tag: 0, start: 100, stop: 10_000},
            Span { tag: 1, start: 20_000, stop: 23_000},
            Span { tag: 2, start: 10_000, stop: 13_000},
            Span { tag: 3, start: 12_000, stop: 13_000},
            Span { tag: 4, start: 20_000, stop: 20_010},
            Span { tag: 5, start: 21_000, stop: 21_010},
            Span { tag: 6, start: 22_000, stop: 22_010},
            Span { tag: 7, start: 5_000, stop: 5_010},
        ];
        assert!(!spans.is_empty(), "expected a non-empty array of trace spans");
        spans.sort_unstable_by_key(|s| s.start);
        let min_start = spans[0].start;
        // TODO check and correct this on first iteration
        let max_stop = spans[spans.len() - 1].stop;
        let window_width = 800u32;
        let window_height = 600u32;
        let scale = (max_stop - min_start) / window_width as u64;
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window("tsc-trace viewer", window_width, window_height)
            .position_centered()
            .opengl()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        Ok((spans, App {
            window_width,
            window_height,
            background_color: Color::RGB(0,0,0),
            colors: vec![
                (255, 0, 0),
                (0, 255, 0),
                (0, 0, 255),
                (255, 255, 0),
                (0, 255, 255),
                (255, 0, 255)
            ].into_iter().map(|c| Color::RGB(c.0, c.1, c.2)).collect(),
            muted_colors: vec![
                (192, 0, 0),
                (0, 192, 0),
                (0, 0, 192),
                (192, 192, 0),
                (0, 192, 192),
                (192, 0, 192)
            ].into_iter().map(|c| Color::RGB(c.0, c.1, c.2)).collect(),
            sdl_context,
            canvas,
            scale,
            span_height: 4,
            span_spacing: 2,
        }))
    }

    fn draw_span(&mut self, span: &Span) {
        let x_sz = self.x_size(span);
        self.canvas.set_draw_color(if x_sz < 1 {
            self.colors[span.tag as usize % self.colors.len()]
        } else {
            self.muted_colors[span.tag as usize % self.muted_colors.len()]
        });
        self.canvas.fill_rect(Rect::new(self.x_pos(span), self.y_pos(span), x_sz, self.span_height as u32))
            .unwrap_or_else(|e| panic!("draw failure {e} for span {span:?}"));
    }

    fn x_size(&self, span: &Span) -> u32 {
        ((span.stop - span.start) / self.scale).try_into()
            .unwrap_or_else(|e| panic!("bad x_size for scale {} span {:?} err {e}", self.scale, span))
    }

    fn x_pos(&self, span: &Span) -> i32 {
        // TODO scrolling viewport
        (span.start / self.scale).try_into()
            .unwrap_or_else(|e| panic!("bad x_pos for scale {} span {:?} err {e}", self.scale, span))
    }

    fn y_pos(&self, span: &Span) -> i32 {
        let tag: i32 = span.tag.try_into()
            .expect("not intended to handle very high span tag cardinality, try filtering / renumbering first: {span}");
        ((self.span_spacing + self.span_height) * tag) + self.span_spacing
    }

    pub fn run(&mut self, spans: Vec<Span>) -> Result<(), String> {
        let mut event_pump = self.sdl_context.event_pump()?;

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::Window{win_event: WindowEvent::Resized(w, h), .. } =>  {
                        self.window_height = h as u32;
                        self.window_width = w as u32;
                    }
                    _ => {}
                }
            }
            
            self.canvas.set_draw_color(self.background_color);
            self.canvas.clear();
            for span in &spans {
                self.draw_span(span);
            }
            self.canvas.present();
            thread::sleep(Duration::new(0, 1_000_000_000u32));
        }

        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let (spans, mut app) = App::new()?;
    println!("{}", app.scale);
    app.run(spans)
}
