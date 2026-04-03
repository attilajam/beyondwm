use crate::Beyond;
use smithay::{
    input::keyboard::ModifiersState,
    input::pointer::{
        AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent,
        GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent,
        GestureSwipeEndEvent, GestureSwipeUpdateEvent, GrabStartData as PointerGrabStartData,
        MotionEvent, PointerGrab, PointerInnerHandle, RelativeMotionEvent,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
};

pub struct MoveCameraGrab {
    pub start_data: PointerGrabStartData<Beyond>,
    pub initial_camera_pos: Point<f64, Logical>,
    pub initial_pointer_pos: Point<f64, Logical>,
}

impl PointerGrab<Beyond> for MoveCameraGrab {
    fn motion(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        _focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &MotionEvent,
    ) {
        handle.motion(data, None, event);

        let output = data.space.outputs().next().unwrap();
        let output_geo = data.space.output_geometry(output).unwrap();
        let current_pointer_pos = event.location - output_geo.loc.to_f64();

        let delta = current_pointer_pos - self.initial_pointer_pos;
        data.canvas_view.camera_pos.x =
            self.initial_camera_pos.x - delta.x / data.canvas_view.camera_scale;

        data.canvas_view.camera_pos.y =
            self.initial_camera_pos.y - delta.y / data.canvas_view.camera_scale;
    }

    fn relative_motion(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &RelativeMotionEvent,
    ) {
        handle.relative_motion(data, focus, event);
    }

    fn button(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        event: &ButtonEvent,
    ) {
        handle.button(data, event);

        // The button is a button code as defined in the
        // Linux kernel's linux/input-event-codes.h header file, e.g. BTN_LEFT.
        const BTN_LEFT: u32 = 0x110;

        let super_held = data
            .seat
            .get_keyboard()
            .map(|kb| kb.modifier_state().ctrl)
            .unwrap_or(false);

        if !super_held || !handle.current_pressed().contains(&BTN_LEFT) {
            // No more buttons are pressed, release the grab.
            handle.unset_grab(self, data, event.serial, event.time, true);
        }
    }

    fn axis(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        details: AxisFrame,
    ) {
        handle.axis(data, details)
    }

    fn frame(&mut self, data: &mut Beyond, handle: &mut PointerInnerHandle<'_, Beyond>) {
        handle.frame(data);
    }

    fn gesture_swipe_begin(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        event: &GestureSwipeBeginEvent,
    ) {
        handle.gesture_swipe_begin(data, event)
    }

    fn gesture_swipe_update(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        event: &GestureSwipeUpdateEvent,
    ) {
        handle.gesture_swipe_update(data, event)
    }

    fn gesture_swipe_end(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        event: &GestureSwipeEndEvent,
    ) {
        handle.gesture_swipe_end(data, event)
    }

    fn gesture_pinch_begin(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        event: &GesturePinchBeginEvent,
    ) {
        handle.gesture_pinch_begin(data, event)
    }

    fn gesture_pinch_update(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        event: &GesturePinchUpdateEvent,
    ) {
        handle.gesture_pinch_update(data, event)
    }

    fn gesture_pinch_end(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        event: &GesturePinchEndEvent,
    ) {
        handle.gesture_pinch_end(data, event)
    }

    fn gesture_hold_begin(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        event: &GestureHoldBeginEvent,
    ) {
        handle.gesture_hold_begin(data, event)
    }

    fn gesture_hold_end(
        &mut self,
        data: &mut Beyond,
        handle: &mut PointerInnerHandle<'_, Beyond>,
        event: &GestureHoldEndEvent,
    ) {
        handle.gesture_hold_end(data, event)
    }

    fn start_data(&self) -> &PointerGrabStartData<Beyond> {
        &self.start_data
    }

    fn unset(&mut self, _data: &mut Beyond) {}
}
