use std::time::Instant;
use std::io;

use std::sync::mpsc::channel;
use std::thread;

use glutin::{
    event::{
        Event,
        WindowEvent,
        KeyboardInput,
        VirtualKeyCode,
        ElementState
    },
    event_loop::{
        ControlFlow,
        EventLoop,
    },
    window::WindowBuilder,
    ContextBuilder,
};

use femtovg::{
    renderer::OpenGl,
    Align,
    Baseline,
    Canvas,
    Color,
    Paint,
    Path,
    Renderer,
};

fn main() {
    let window_size = glutin::dpi::PhysicalSize::new(1000, 670);
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_inner_size(window_size)
        .with_resizable(false)
        .with_title("Gradient test");

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let renderer = OpenGl::new(|s| windowed_context.get_proc_address(s) as *const _).expect("Cannot create renderer");
    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    canvas.set_size(
        window_size.width as u32,
        window_size.height as u32,
        windowed_context.window().scale_factor() as f32,
    );
    canvas
        .add_font("/home/rne/Pobrane/cataclysmdda-0.D/data/font/fixedsys.ttf")
        .expect("Cannot add font"); 

    let start = Instant::now();
    let mut prevt = start;

    let mut perf = PerfGraph::new();

    let (sender, reciever) = channel();

    let mut input_text = String::new();
    let mut line_starts : Vec<usize> = vec![0];
    
    thread::spawn(move|| {
        let input = io::stdin();
        loop {
            let mut buffer = String::new();
            match input.read_line(&mut buffer) {
                Ok(_n) => sender.send(buffer).unwrap_or(()),
                Err(_) => break
            }
        }
    });

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(*physical_size);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Q),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,  
                _ => (),
            },
            Event::RedrawRequested(_) => {
                let dpi_factor = windowed_context.window().scale_factor();
                let size = windowed_context.window().inner_size();
                canvas.set_size(size.width as u32, size.height as u32, dpi_factor as f32);
                canvas.clear_rect(0, 0, size.width as u32, size.height as u32, Color::rgbf(0.9, 0.9, 0.9));

                let now = Instant::now();
                let dt = (now - prevt).as_secs_f32();
                prevt = now;

                perf.update(dt);

                match reciever.try_recv() {
                    Ok(line) => {
                        line_starts.push(input_text.len());
                        if line_starts.len()>200 {
                            line_starts.remove(0);
                            line_starts.remove(0);
                            line_starts.remove(0);
                            line_starts.remove(0);
                            line_starts.remove(0);
                            line_starts.remove(0);
                            line_starts.remove(0);
                            line_starts.remove(0);
                            line_starts.remove(0);
                            line_starts.remove(0);
                        }
                        input_text.push_str(&line);
                    },
                    Err(_) => ()
                }

                draw_stuff(&mut canvas, &input_text, &line_starts);

                canvas.save();
                canvas.reset();
                perf.render(&mut canvas, 5.0, 5.0);
                canvas.restore();

                canvas.flush();
                windowed_context.swap_buffers().unwrap();
            }
            Event::MainEventsCleared => windowed_context.window().request_redraw(),
            _ => (),
        }
    });
}

fn draw_stuff<T: Renderer>(canvas: &mut Canvas<T>, text: &String, line_starts: &Vec<usize>) {
    let mut text_paint = Paint::color(Color::black());
        text_paint.set_font_size(14.0);

    canvas.save();

    let display_line_start : usize = if line_starts.len() < 10 {
        line_starts[0]
    } else {
        line_starts[line_starts.len()-10]
    };
    let text = text.get(display_line_start..).unwrap();
    
    canvas.translate(0.0, 50.0);
    for line in text.split("\n") {
        let _ = canvas.fill_text(0.0, 0.0, line, text_paint);
        canvas.translate(0.0, 10.0);
    }
    canvas.restore();
}

struct PerfGraph {
    history_count: usize,
    values: Vec<f32>,
    head: usize,
}

impl PerfGraph {
    fn new() -> Self {
        Self {
            history_count: 100,
            values: vec![0.0; 100],
            head: Default::default(),
        }
    }

    fn update(&mut self, frame_time: f32) {
        self.head = (self.head + 1) % self.history_count;
        self.values[self.head] = frame_time;
    }

    fn get_average(&self) -> f32 {
        self.values.iter().map(|v| *v).sum::<f32>() / self.history_count as f32
    }

    fn render<T: Renderer>(&self, canvas: &mut Canvas<T>, x: f32, y: f32) {
        let avg = self.get_average();

        let w = 200.0;
        let h = 35.0;

        let mut path = Path::new();
        path.rect(x, y, w, h);
        canvas.fill_path(&mut path, Paint::color(Color::rgba(0, 0, 0, 128)));

        let mut path = Path::new();
        path.move_to(x, y + h);

        for i in 0..self.history_count {
            let mut v = 1.0 / (0.00001 + self.values[(self.head + i) % self.history_count]);
            if v > 80.0 {
                v = 80.0;
            }
            let vx = x + (i as f32 / (self.history_count - 1) as f32) * w;
            let vy = y + h - ((v / 80.0) * h);
            path.line_to(vx, vy);
        }

        path.line_to(x + w, y + h);
        canvas.fill_path(&mut path, Paint::color(Color::rgba(255, 192, 0, 128)));

        let mut text_paint = Paint::color(Color::rgba(240, 240, 240, 255));
        text_paint.set_font_size(12.0);
        let _ = canvas.fill_text(x + 5.0, y + 13.0, "Frame time", text_paint);

        let mut text_paint = Paint::color(Color::rgba(240, 240, 240, 255));
        text_paint.set_font_size(14.0);
        text_paint.set_text_align(Align::Right);
        text_paint.set_text_baseline(Baseline::Top);
        let _ = canvas.fill_text(x + w - 5.0, y, &format!("{:.2} FPS", 1.0 / avg), text_paint);

        let mut text_paint = Paint::color(Color::rgba(240, 240, 240, 200));
        text_paint.set_font_size(12.0);
        text_paint.set_text_align(Align::Right);
        text_paint.set_text_baseline(Baseline::Alphabetic);
        let _ = canvas.fill_text(x + w - 5.0, y + h - 5.0, &format!("{:.2} ms", avg * 1000.0), text_paint);
    }
}
