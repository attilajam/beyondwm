use std::{io::empty, time::Duration};

use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker,
            element::{
                AsRenderElements, surface::WaylandSurfaceRenderElement, utils::RescaleRenderElement,
            },
            gles::GlesRenderer,
        },
        winit::{self, WinitEvent},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Point, Rectangle, Scale, Transform},
};

use crate::Beyond;

pub fn init_winit(
    event_loop: &mut EventLoop<Beyond>,
    state: &mut Beyond,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut backend, winit) = winit::init()?;

    let mode = Mode {
        size: backend.window_size(),
        refresh: 60_000,
    };

    let output = Output::new(
        "winit".to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Smithay".into(),
            model: "Winit".into(),
            serial_number: "Unknown".into(),
        },
    );
    let _global = output.create_global::<Beyond>(&state.display_handle);
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);

    let camera_pos = state.canvas_view.camera_pos.to_i32_floor();
    state
        .space
        .map_output(&output, (camera_pos.x, camera_pos.y));

    let mut damage_tracker = OutputDamageTracker::from_output(&output);

    event_loop
        .handle()
        .insert_source(winit, move |event, _, state| {
            match event {
                WinitEvent::Resized { size, .. } => {
                    output.change_current_state(
                        Some(Mode {
                            size,
                            refresh: 60_000,
                        }),
                        None,
                        None,
                        None,
                    );
                }
                WinitEvent::Input(event) => state.process_input_event(event),
                WinitEvent::Redraw => {
                    let camera_pos = state.canvas_view.camera_pos.to_i32_floor();
                    state
                        .space
                        .map_output(&output, (camera_pos.x, camera_pos.y));
                    let size = backend.window_size();
                    let damage = Rectangle::from_size(size);

                    {
                        let (renderer, mut framebuffer) = backend.bind().unwrap();
                        let scale = Scale::from(state.canvas_view.camera_scale);
                        let origin = Point::from((0, 0));
                        let custom_elements: Vec<
                            RescaleRenderElement<WaylandSurfaceRenderElement<GlesRenderer>>,
                        > = state
                            .space
                            .elements()
                            .flat_map(|window| {
                                let loc = state.space.element_location(window).unwrap_or_default();
                                let logical_loc = state.canvas_view.to_screen(loc.to_f64());
                                let physical_loc = logical_loc.to_i32_floor().to_physical(1);

                                window
                                    .render_elements(renderer, physical_loc, scale, 1.0)
                                    .into_iter()
                                    .map(move |e| {
                                        RescaleRenderElement::from_element(e, origin, scale)
                                    })
                            })
                            .collect();

                        damage_tracker
                            .render_output(
                                renderer,
                                &mut framebuffer,
                                0,
                                &custom_elements,
                                [0.1, 0.1, 0.1, 1.0],
                            )
                            .unwrap();
                    }
                    backend.submit(Some(&[damage])).unwrap();

                    state.space.elements().for_each(|window| {
                        window.send_frame(
                            &output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        )
                    });

                    state.space.refresh();
                    state.popups.cleanup();
                    let _ = state.display_handle.flush_clients();

                    // Ask for redraw to schedule new frame.
                    backend.window().request_redraw();
                }
                WinitEvent::CloseRequested => {
                    state.loop_signal.stop();
                }
                _ => (),
            };
        })?;

    Ok(())
}
