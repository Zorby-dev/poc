use std::{path::Path, fs, sync::{Arc, mpsc}, thread, time::Duration};

use clap::Parser;
use pemu::{cpu::{CPU, Port, RAM, Keyboard, Floppy, sti::{self, STI, STIInput, STIOutput, STIPrinter}, keyboard_output, KeyboardOutput}, cli::Cli};
use pixels::{SurfaceTexture, Pixels};
use winit::{event_loop::EventLoopBuilder, dpi::LogicalSize, window::WindowBuilder, event::{Event, WindowEvent, ElementState}, platform::windows::EventLoopBuilderExtWindows};

fn load_image(image: &Path, ram_size: usize) -> Result<Vec<u8>, String> {
    let image = fs::read(image)
        .map_err(|e| format!("unable to open `{}`: {}", image.display(), e))?;

    if image.len() != ram_size {
        return Err(format!("image size does not match RAM size"));
    }

    return Ok(image)
}

fn init_ram(image: Option<&Path>, size: usize) -> Result<RAM, String> {
    let image = match image {
        Some(image) => load_image(image, size)?,
        None => vec![0; size]
    };

    Ok(RAM::from(image))
}

fn start_virtual_screen(keyboard_output: KeyboardOutput, sti_input: STIInput) {
    let mut sti_printer = STIPrinter::new(sti_input);

    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

    let window = {
        let size = LogicalSize::new(
            sti::WIDTH * sti::SCALE,
            sti::HEIGHT * sti::SCALE);
        WindowBuilder::new()
            .with_title("PEMU Virtual Screen")
            .with_inner_size(size.clone())
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let surface_texture = SurfaceTexture::new(
            sti::WIDTH * sti::SCALE,
            sti::HEIGHT * sti::SCALE, &window);
        Pixels::new(sti::WIDTH, sti::HEIGHT, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::Resized(size) => pixels.resize_surface(size.width, size.height).unwrap(),
                WindowEvent::KeyboardInput { input, .. } => {
                    let mut lock = keyboard_output.lock();

                    *lock = match input.state {
                        ElementState::Pressed => input.scancode as u8,
                        ElementState::Released => 0
                    };
                },
                _ => ()
            },
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                sti_printer.update(pixels.frame_mut());
                pixels.render().unwrap();
            },
            _ => ()
        }
    });
}

fn start_computer(ram_image: Option<&Path>) -> Result<CPU, String> {
    let ram_size = 252;

    let keyboard_output = keyboard_output();

    let (sti_output, sti_input): (STIOutput, STIInput) = mpsc::channel();

    let ports = vec![
        Port::new(  0, ram_size, init_ram(ram_image, ram_size)?),
        Port::new(252,      253, Keyboard::new(Arc::clone(&keyboard_output))),
        Port::new(253,      255, Floppy::new()),
        Port::new(255,      256, STI::new(sti_output))
    ];

    thread::spawn(move ||
        start_virtual_screen(keyboard_output, sti_input));

    let cpu = CPU::new(ports);

    Ok(cpu)
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let ram_image = cli.image.as_deref();

    let mut cpu = start_computer(ram_image)?;

    loop {
        cpu.step();
        spin_sleep::sleep(Duration::from_nanos(100));
    }
}