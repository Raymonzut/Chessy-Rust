use glutin::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};

use femtovg::{
    renderer::OpenGl, Canvas, Color, ImageFlags, Paint, Path, PixelFormat, RenderTarget,
};

fn main() {
    let window_size = glutin::dpi::PhysicalSize::new(1000, 600);
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_inner_size(window_size)
        .with_resizable(false)
        .with_title("Chessy");

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let renderer =
        OpenGl::new_from_glutin_context(&windowed_context).expect("Cannot create renderer");
    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    canvas.set_size(
        window_size.width as u32,
        window_size.height as u32,
        windowed_context.window().scale_factor() as f32,
    );

    // Prepare the 8x8 chess grid
    let grid_size: usize = (u32::min(window_size.width, window_size.height) / 8) as usize;
    let image_id = canvas
        .create_image_empty(
            8 * grid_size + 1,
            8 * grid_size + 1,
            PixelFormat::Rgba8,
            ImageFlags::empty(),
        )
        .unwrap();
    canvas.save();
    canvas.reset();
    if let Ok(size) = canvas.image_size(image_id) {
        canvas.set_render_target(RenderTarget::Image(image_id));
        canvas.clear_rect(0, 0, size.0 as u32, size.1 as u32, Color::rgb(0, 0, 0));
        for x in 0..8 {
            for y in 0..8 {
                canvas.clear_rect(
                    (x * grid_size + 1) as u32,
                    (y * grid_size + 1) as u32,
                    (grid_size - 1) as u32,
                    (grid_size - 1) as u32,
                    cell_color(x, y),
                );
            }
        }
    }
    canvas.restore();

    let mut zoom = 0;

    eprintln!("Scroll to change zoom");

    let mut swap_directions = false;

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(*physical_size);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::ModifiersChanged(modifiers) => {
                    swap_directions = modifiers.shift();
                }
                WindowEvent::MouseWheel {
                    device_id: _,
                    delta,
                    ..
                } => match delta {
                    glutin::event::MouseScrollDelta::LineDelta(x, y) => {
                        if swap_directions {
                            zoom += *x as i32;
                        } else {
                            zoom += *y as i32;
                        }
                    }
                    _ => (),
                },
                WindowEvent::MouseInput {
                    device_id: _,
                    state: ElementState::Pressed,
                    ..
                } => (),
                _ => (),
            },
            Event::RedrawRequested(_) => {
                let dpi_factor = windowed_context.window().scale_factor();
                let window_size = windowed_context.window().inner_size();
                canvas.set_size(
                    window_size.width as u32,
                    window_size.height as u32,
                    dpi_factor as f32,
                );
                canvas.clear_rect(
                    0,
                    0,
                    window_size.width as u32,
                    window_size.height as u32,
                    Color::rgbf(0.2, 0.2, 0.2),
                );

                canvas.save();
                canvas.reset();

                let zoom = (zoom as f32 / 40.0).exp();
                canvas.translate(
                    window_size.width as f32 / 2.0,
                    window_size.height as f32 / 2.0,
                );
                canvas.scale(zoom, zoom);
                canvas.translate(
                    window_size.width as f32 / -2.0,
                    window_size.height as f32 / -2.0,
                );

                if let Ok(size) = canvas.image_size(image_id) {
                    let width = f32::max(1.0, size.0 as f32 * zoom);
                    let height = f32::max(1.0, size.1 as f32 * zoom);
                    let x = window_size.width as f32 / 2.0;
                    let y = window_size.height as f32 / 2.0;

                    let mut path = Path::new();
                    path.rect(x - width / 2.0, y - height / 2.0, width, height);

                    // Get the bounding box of the path so that we can stretch
                    // the paint to cover it exactly:
                    let bbox = canvas.path_bbox(&mut path);

                    // Now we need to apply the current canvas transform
                    // to the path bbox:
                    let a = canvas
                        .transform()
                        .inversed()
                        .transform_point(bbox.minx, bbox.miny);
                    let b = canvas
                        .transform()
                        .inversed()
                        .transform_point(bbox.maxx, bbox.maxy);

                    canvas.fill_path(
                        &mut path,
                        Paint::image(image_id, a.0, a.1, b.0 - a.0, b.1 - a.1, 0f32, 1f32),
                    );
                }

                canvas.restore();

                canvas.flush();
                windowed_context.swap_buffers().unwrap();
            }
            Event::MainEventsCleared => windowed_context.window().request_redraw(),
            _ => (),
        }
    });
}

fn cell_color(x: usize, y: usize) -> Color {
    let p1_color = Color::rgb(225, 225, 225);
    let p2_color = Color::rgb(10, 10, 10);

    match (x + y) % 2 {
        0 => p1_color,
        _ => p2_color,
    }
}
