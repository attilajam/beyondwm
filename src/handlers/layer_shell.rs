use smithay::{
    delegate_layer_shell,
    desktop::{LayerSurface, WindowSurfaceType, layer_map_for_output},
    output::Output,
    reexports::wayland_server::protocol::{
        wl_output::WlOutput,
        wl_surface::{self, WlSurface},
    },
    utils::SERIAL_COUNTER,
    wayland::{
        compositor::{get_parent, with_states},
        shell::wlr_layer::{
            Layer, LayerSurface as WlrLayerSurface, LayerSurfaceData, WlrLayerShellHandler,
            WlrLayerShellState,
        },
    },
};

use crate::Beyond;

impl WlrLayerShellHandler for Beyond {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.wlr_layer_shell_state
    }
    fn new_layer_surface(
        &mut self,
        surface: WlrLayerSurface,
        wl_output: Option<WlOutput>,
        _layer: Layer,
        namespace: String,
    ) {
        let output = if let Some(wl_output) = &wl_output {
            Output::from_resource(wl_output)
        } else {
            self.space.outputs().next().cloned()
        };
        let Some(output) = output else {
            surface.send_close();
            return;
        };

        let wl_surface = surface.wl_surface().clone();
        let is_new = self.unmapped_layer_surfaces.insert(wl_surface);
        assert!(is_new);

        let mut map = layer_map_for_output(&output);
        map.map_layer(&LayerSurface::new(surface, namespace))
            .unwrap();
    }
    fn layer_destroyed(&mut self, surface: WlrLayerSurface) {
        let wl_surface = surface.wl_surface();
        self.unmapped_layer_surfaces.remove(wl_surface);
        self.mapped_layer_surface = Option::<WlSurface>::None;
        let output = if let Some((output, mut map, layer)) = self.space.outputs().find_map(|o| {
            let map = layer_map_for_output(o);
            let layer = map
                .layers()
                .find(|&layer| layer.layer_surface() == &surface)
                .cloned();
            layer.map(|layer| (o.clone(), map, layer))
        }) {
            map.unmap_layer(&layer);
            Some(output)
        } else {
            None
        };
        if let Some(output) = output {
            layer_map_for_output(&output).arrange();
            self.space.refresh();
        }
    }
}
delegate_layer_shell!(Beyond);

impl Beyond {
    pub fn layer_shell_handle_commit(&mut self, surface: &WlSurface) -> bool {
        let mut root_surface = surface.clone();
        while let Some(parent) = get_parent(&root_surface) {
            root_surface = parent;
        }

        let output = self
            .space
            .outputs()
            .find(|o| {
                let map = layer_map_for_output(o);
                map.layer_for_surface(&root_surface, WindowSurfaceType::TOPLEVEL)
                    .is_some()
            })
            .cloned();
        let Some(output) = output else {
            return false;
        };

        if surface != &root_surface {
            // This is an unsync layer-shell subsurface.
            self.space.refresh();
            return true;
        }

        let mut map = layer_map_for_output(&output);

        // Arrange the layers before sending the initial configure to respect any size the
        // client may have sent.
        map.arrange();

        let layer = map
            .layer_for_surface(surface, WindowSurfaceType::TOPLEVEL)
            .unwrap();

        // An unmapped surface remains unmapped. If we haven't sent an initial configure
        // yet, we should do so.
        let initial_configure_sent = with_states(surface, |states| {
            states
                .data_map
                .get::<LayerSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });
        if !initial_configure_sent {
            layer.layer_surface().send_configure();
        } else {
            let was_unmapped = self.unmapped_layer_surfaces.remove(surface);
            if was_unmapped {
                println!("layer surface mapped")
            }
            let keyboard = self.seat.get_keyboard().unwrap();
            let serial = SERIAL_COUNTER.next_serial();
            keyboard.set_focus(self, Some(layer.wl_surface().clone()), serial);
            self.mapped_layer_surface = Some(layer.wl_surface().clone());
        }
        // If we already sent an initial configure, then map.arrange() above had just sent
        // it a new configure, if needed.

        drop(map);

        // This will call queue_redraw() inside.
        self.space.refresh();

        true
    }
}

pub fn handle_commit(beyond: &mut Beyond, surface: &WlSurface) {
    beyond.layer_shell_handle_commit(surface);
}
