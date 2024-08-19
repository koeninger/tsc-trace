pub mod config;

use bytemuck::Pod;
use bytemuck::Zeroable;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

const FRAME: u32 = 33_333_333;

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Span {
    tag: u64,
    start: u64,
    stop: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct Area {
    y_start: u64,
    x_start: i32,
    y_stop: u64,
    x_stop: i32,
    tag_data: Span,
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    x: i32,
    y: i32,
}

pub struct App {
    window_width: u32,
    background_color: Color,
    draw_zones: Vec<Area>,
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
    min_start: u64,
    max_stop: u64,
    scroll: i32,
    texture_creator: TextureCreator<WindowContext>,
}

impl App {
    pub fn new(filled_spans: &mut Vec<Span>) -> Result<App, String> {
        let config = config::config();
        let mut spans: Vec<Span> = vec![];
        for span in filled_spans {
            spans.push(*span);
        }
        assert!(
            !spans.is_empty(),
            "expected a non-empty array of trace spans"
        );
        spans.sort_unstable_by_key(|s| s.start);
        let min_start = spans[0].start;
        let max_stop = spans.iter().max_by_key(|s| s.stop).unwrap().stop;
        let draw_zones: Vec<Area> = vec![];
        let window_width = config.window_width;
        let window_height = config.window_height;
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let scale = (max_stop - min_start) / window_width as u64;
        let window = video_subsystem
            .window(
                &env::args().collect::<Vec<String>>()[1],
                window_width,
                window_height,
            )
            .position_centered()
            .opengl()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();

        Ok(App {
            texture_creator,
            draw_zones,
            window_width,
            background_color: Color::RGB(192, 192, 192),
            colors: vec![
                (255, 0, 0),
                (0, 255, 0),
                (0, 0, 255),
                (255, 255, 0),
                (0, 255, 255),
                (255, 0, 255),
            ]
            .into_iter()
            .map(|c| Color::RGB(c.0, c.1, c.2))
            .collect(),
            muted_colors: vec![
                (192, 0, 0),
                (0, 192, 0),
                (0, 0, 192),
                (192, 192, 0),
                (0, 192, 192),
                (192, 0, 192),
            ]
            .into_iter()
            .map(|c| Color::RGB(c.0, c.1, c.2))
            .collect(),
            sdl_context,
            canvas,
            scale,
            span_height: config.span_height,
            span_spacing: config.span_spacing,
            min_start,
            max_stop,
            scroll: 0,
        })
    }

    fn draw_span(&mut self, span: &Span) {
        let x_sz = self.x_size(span);
        let scrolled_x = self.x_pos(span).saturating_sub(self.scroll);
        //preventing drawing if span would be outside of the window
        if scrolled_x
            < self
                .window_width
                .try_into()
                .expect("Window width couldn't be parsed")
            && (scrolled_x + x_sz as i32) > 0
        {
            //using lighter color palette if multiple small spans could occupy the same pixel
            self.canvas.set_draw_color(if x_sz < 1 {
                self.colors[span.tag as usize % self.colors.len()]
            } else {
                self.draw_zones.push(Area {
                    y_start: self.y_pos(span) as u64,
                    x_start: scrolled_x,
                    y_stop: (self.y_pos(span) + self.span_height) as u64,
                    x_stop: (scrolled_x + x_sz as i32),
                    tag_data: *span,
                });
                self.muted_colors[span.tag as usize % self.muted_colors.len()]
            });
            self.canvas
                .fill_rect(Rect::new(
                    scrolled_x,
                    self.y_pos(span),
                    x_sz,
                    (self.span_height) as u32,
                ))
                .unwrap_or_else(|e| panic!("draw failure {e} for span {span:?}"));
        }
    }

    fn x_size(&self, span: &Span) -> u32 {
        ((span.stop - span.start) / (self.scale + 1))
            .try_into()
            .unwrap_or_else(|e| {
                panic!(
                    "bad x_size for scale {} span {:?} err {e}",
                    self.scale, span
                )
            })
    }

    fn x_pos(&self, span: &Span) -> i32 {
        ((span.start - self.min_start) / (self.scale + 1))
            .try_into()
            .unwrap_or_else(|_| i32::MAX)
    }

    fn y_pos(&self, span: &Span) -> i32 {
        let tag: i32 = span.tag.try_into()
            .expect("not intended to handle very high span tag cardinality, try filtering / renumbering first: {span}");
        ((self.span_spacing + self.span_height) * tag) + self.span_spacing
    }

    fn draw_text(
        canvas: &mut WindowCanvas,
        texture_creator: &TextureCreator<WindowContext>,
        font: &sdl2::ttf::Font,
        x: i32,
        y: i32,
        tag_data: &Span,
    ) -> Result<(), String> {
        let config = config::config();
        let tag_text: String;
        match config.tag_names {
            Some(hashmap) => {
                tag_text = hashmap
                    .get(&tag_data.tag)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| tag_data.tag.to_string());
            }
            None => {
                tag_text = tag_data.tag.to_string();
            }
        }
        let tag_text = format!("{0},{1}", tag_text, (tag_data.stop - tag_data.start));

        let surface = font
            .render(&tag_text)
            .blended(Color::RGBA(0, 0, 0, 128))
            .map_err(|e| e.to_string())?;

        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.fill_rect(Rect::new(x, y, (tag_text.len() * 20) as u32, 50_u32))?;

        let target = Rect::new(x, y, (tag_text.len() * 20) as u32, 50_u32);
        canvas.copy(&texture, None, Some(target))?;

        Ok(())
    }

    pub fn run(&mut self, spans: Vec<Span>, font: Font<'_, 'static>) -> Result<(), String> {
        let mut event_pump = self.sdl_context.event_pump()?;
        let mut prev_keycount: i32 = 0;
        let mut keycount: i32 = 0;
        let mut draw_x = 0;
        let mut draw_y = 0;
        let mut draw_data = Span {
            tag: 0,
            start: 0,
            stop: 0,
        };
        let mut all_spans_map: HashMap<i32, i32> = HashMap::new();
        let mut most_recent_spans: Vec<Position> = vec![];

        //getting each y position to be drawn
        for span in &spans {
            all_spans_map.insert(self.y_pos(span), -1);
        }
        //converting the HashMap to a Vec for increased performance on small numbers of tags
        for (y, x) in all_spans_map {
            most_recent_spans.push(Position { x, y });
        }

        'running: loop {
            let loop_time = Instant::now();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        keycount += 1;
                        match keycode {
                            //zoom out
                            Keycode::Q => {
                                self.scale = self
                                    .scale
                                    .saturating_add((keycount * keycount).try_into().unwrap())
                            }
                            //zoom in
                            Keycode::W => {
                                self.scale = self
                                    .scale
                                    .saturating_sub((keycount * keycount).try_into().unwrap())
                            }
                            //reset
                            Keycode::E => {
                                self.scale =
                                    (self.max_stop - self.min_start) / (self.window_width as u64)
                            }
                            Keycode::S => self.scroll = self.scroll.saturating_add(keycount), //scroll right
                            Keycode::A => self.scroll = self.scroll.saturating_sub(keycount), //scroll left
                            Keycode::D => self.scroll = 0, //reset
                            _ => {}
                        }
                    }
                    Event::MouseButtonDown { x, y, .. } => {
                        for zone in &self.draw_zones {
                            if zone.x_start < x
                                && zone.x_stop > x
                                && zone.y_start < y.try_into().unwrap()
                                && zone.y_stop > y.try_into().unwrap()
                            {
                                draw_x = x;
                                draw_y = y;
                                draw_data = zone.tag_data;
                                println!("{0:?}", draw_data);
                            }
                        }
                    }
                    Event::MouseButtonUp { .. } => {
                        draw_x = 0;
                        draw_y = 0;
                    }
                    Event::Window {
                        win_event: WindowEvent::Resized(w, ..),
                        ..
                    } => {
                        self.window_width = w as u32;
                    }
                    _ => {}
                }
            }
            if prev_keycount == keycount {
                keycount = 0;
            }
            prev_keycount = keycount;

            self.canvas.set_draw_color(self.background_color);
            self.canvas.clear();
            self.draw_zones.clear();
            for span in &spans {
                let pos = Position {
                    x: self.x_pos(span),
                    y: self.y_pos(span),
                };
                //preventing draw_span from being called if it would draw to an already filled position
                for buffer_pos in &mut most_recent_spans {
                    if pos.y == buffer_pos.y {
                        if pos.x != buffer_pos.x || self.x_size(span) > 0 {
                            self.draw_span(span);
                            *buffer_pos = pos;
                        }
                        break;
                    }
                }
            }
            if (draw_x > 0) && (draw_y > 0) {
                Self::draw_text(
                    &mut self.canvas,
                    &self.texture_creator,
                    &font,
                    draw_x,
                    draw_y,
                    &draw_data,
                )?;
            }

            self.canvas.present();

            thread::sleep(Duration::new(
                0,
                FRAME.saturating_sub(
                    loop_time
                        .elapsed()
                        .as_nanos()
                        .try_into()
                        .unwrap_or(u32::MAX),
                ),
            ));
        }

        Ok(())
    }
}

pub fn load_args(mut args: Vec<String>) -> Vec<Span> {
    let mut spans = vec![];
    let config = config::config();
    match args.len() {
        1 => {panic!("Command line arguments were not provided. Format: (file path) (span range start) (span range stop) (tag range start) (tag range stop).")},
        //loading default arguments as long as the file path is provided
        2 => {
            args.push(config.default_args[0].clone());
            spans = load_args(args);
        },
        3 => {
            args.push(config.default_args[1].clone());
            spans = load_args(args);
        },
        4 => {
            args.push(config.default_args[2].clone());
            spans = load_args(args);
        },
        5 => {
            println!("One or more arguments not provided. Running with defaults for missing values.");
            args.push(config.default_args[3].clone());
            spans = load_args(args);
        },
        6 =>{
            let mut file = File::open(&args[1]).expect("failed to open file");
            let mut buffer = [0; 24];
            let span_start = args[2].parse::<u64>().expect("Could not parse span range start");
            let span_stop = args[3].parse::<u64>().expect("Could not parse span range stop");
            let tag_start = args[4].parse::<u64>().expect("Could not parse tag range start");
            let tag_stop = args [5].parse::<u64>().expect("Could not parse tag range stop");

            loop{
                file.read_exact(&mut buffer).expect("failed to fill buffer");
                let mut s: Span = bytemuck::pod_read_unaligned(&buffer);
                if s.tag >= tag_start
                    && s.tag <= tag_stop
                {
                    while s.start <= span_stop{
                        if s.start >= span_start
                            && s.tag >= tag_start
                            && s.tag <= tag_stop
                        {
                            spans.push(s);
                        }
                        match file.read_exact(&mut buffer) {
                            Ok(..) => {
                                s = bytemuck::pod_read_unaligned(&buffer);
                            }
                            Err(..) => {
                                break;
                            }
                        }
                    }
                    break;
                }
            }
        },
        _ => panic!("Command line arguments could not be parsed. Format: (file path) (span range start) (span range stop) (tag range start) (tag range stop)."),
    }
    spans
}

pub fn main() -> Result<(), String> {
    let mut filled_spans = load_args(env::args().collect::<Vec<String>>());
    let mut app = App::new(&mut filled_spans)?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let font_path: &Path = Path::new(&"fonts/Opensans-Regular.ttf");
    let font = ttf_context.load_font(font_path, 128)?;
    app.run(filled_spans, font)
}
