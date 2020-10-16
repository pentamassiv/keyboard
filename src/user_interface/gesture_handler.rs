// Imports from other crates
use relm::Relm;
use std::time::Instant;

// Imports from other modules
use crate::keyboard::{Interaction, SwipeAction, TapDuration, TapMotion};
use crate::user_interface::{Msg, Win};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub time: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureSignal {
    DragBegin,
    DragUpdate,
    DragEnd,
    LongPress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GestureInterpretation {
    NoGesture,
    LongPress,
    Swipe,
}

#[derive(Clone)]
pub struct GestureModel {
    relm: Relm<Win>,
    prev_interpretation: GestureInterpretation,
    swipe_path: Vec<Point>,
}

impl GestureModel {
    pub fn new(relm: Relm<Win>) -> GestureModel {
        let prev_interpretation = GestureInterpretation::NoGesture;
        let swipe_path = Vec::new();
        GestureModel {
            relm,
            prev_interpretation,
            swipe_path,
        }
    }

    pub fn handle(&mut self, x: f64, y: f64, input: GestureSignal) {
        let interaction = match input {
            GestureSignal::DragBegin => Interaction::Tap(TapDuration::Short, TapMotion::Press),
            GestureSignal::LongPress => {
                self.prev_interpretation = GestureInterpretation::LongPress;
                Interaction::Tap(TapDuration::Long, TapMotion::Press)
            }
            GestureSignal::DragUpdate => {
                self.swipe_path.push(Point {
                    x,
                    y,
                    time: Instant::now(),
                });
                match self.prev_interpretation {
                    GestureInterpretation::NoGesture | GestureInterpretation::LongPress => {
                        self.prev_interpretation = GestureInterpretation::Swipe;
                        Interaction::Swipe(SwipeAction::Begin)
                    }
                    GestureInterpretation::Swipe => Interaction::Swipe(SwipeAction::Update),
                }
            }
            GestureSignal::DragEnd => {
                let new_interaction = match self.prev_interpretation {
                    GestureInterpretation::NoGesture => {
                        Interaction::Tap(TapDuration::Short, TapMotion::Release)
                    }
                    GestureInterpretation::LongPress => {
                        Interaction::Tap(TapDuration::Long, TapMotion::Release)
                    }
                    GestureInterpretation::Swipe => Interaction::Swipe(SwipeAction::Finish),
                };
                self.swipe_path = Vec::new();
                self.prev_interpretation = GestureInterpretation::NoGesture;
                new_interaction
            }
        };
        self.relm
            .stream()
            .emit(Msg::Interaction((x, y), interaction));
    }

    #[cfg(feature = "gesture")]
    pub fn get_swipe_path(&self) -> Vec<Point> {
        self.swipe_path.clone()
    }
}
