// Imports from other crates
use std::time::Instant;

// Imports from other modules
use crate::keyboard::{Interaction, SwipeAction, TapDuration, TapMotion};

/// Coordinate and time of an interaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub time: Instant,
}

/// Signals that can be sent to the GestureModel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureSignal {
    DragBegin,
    DragUpdate,
    DragEnd,
    LongPress,
}

/// Internal type needed to make the interpretation of the signals simpler
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GestureInterpretation {
    NoGesture,
    LongPress,
    Swipe,
}

#[derive(Clone)]
/// Converts the signals of user input into 'Interaction's the keyboard can handle
/// and stores the points of the swipe path
pub struct GestureModel {
    prev_interpretation: GestureInterpretation,
    swipe_path: Vec<Point>,
}

impl GestureModel {
    /// Create a new GestureModel
    pub fn new() -> GestureModel {
        let prev_interpretation = GestureInterpretation::NoGesture;
        let swipe_path = Vec::new();
        GestureModel {
            prev_interpretation,
            swipe_path,
        }
    }

    /// Converts the signal to an Interaction
    pub fn convert_to_interaction(
        &mut self,
        x: f64,
        y: f64,
        input: GestureSignal,
    ) -> ((f64, f64), Interaction) {
        let interaction = match input {
            // If the signal was a DragBegin, the interaction was a short press
            GestureSignal::DragBegin => Interaction::Tap(TapDuration::Short, TapMotion::Press),
            // If the signal was a LongPress, the interaction was a long press
            GestureSignal::LongPress => {
                self.prev_interpretation = GestureInterpretation::LongPress;
                Interaction::Tap(TapDuration::Long, TapMotion::Press)
            }
            // If the signal was a DragUpdate, the interaction depends on the previous interpretation of a signal
            // If the previous interpretation was a swipe already, the interaction is a SwipeUpdate
            // Otherwise it is the beginning of a swipe
            // In all cases it adds the coordinates to the swipe path
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
            // If the signal was a DragEnd, the swipe path is cleared
            // and the returned interaction depends on the previous interpretation of a signal
            GestureSignal::DragEnd => {
                // Clear swipe path
                self.swipe_path = Vec::new();
                let new_interaction = match self.prev_interpretation {
                    // If no gesture was detected, it was a short release
                    GestureInterpretation::NoGesture => {
                        Interaction::Tap(TapDuration::Short, TapMotion::Release)
                    }
                    // If a long press was previously detected, it was a long release
                    GestureInterpretation::LongPress => {
                        Interaction::Tap(TapDuration::Long, TapMotion::Release)
                    }
                    // And if the previously detected gesture was a swipe, the returned interaction is a 'SwipeFinish'
                    GestureInterpretation::Swipe => Interaction::Swipe(SwipeAction::Finish),
                };
                self.prev_interpretation = GestureInterpretation::NoGesture;
                new_interaction
            }
        };
        ((x, y), interaction)
    }

    #[cfg(feature = "gesture")]
    /// Returns the swipe path
    pub fn get_swipe_path(&self) -> Vec<Point> {
        self.swipe_path.clone()
    }
}
