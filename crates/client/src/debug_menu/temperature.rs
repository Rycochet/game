//! Cycle Temperature: the snow line falling all the way and rising again.

use bevy::prelude::*;
use shared::protocol::ClientMessage;

use super::TemperatureCycle;
use crate::common::{NORMAL_SNOW_LINE, Weather};
use crate::connection::ToServer;

/// How fast the snow line moves during the temperature cycle, in height units
/// per second (the prototype moved 1 unit every 166 ms).
const SNOW_LINE_SPEED: f64 = 6.0;

/// Moves the snow line by asking the server, whose word `Weather` holds, as it
/// decides where ice can be walked on (dev tools: only the embedded server
/// allows it). `line` keeps the fractions between whole heights.
pub(super) fn temperature_cycle(
    time: Res<Time>,
    mut cycle: ResMut<TemperatureCycle>,
    weather: Res<Weather>,
    mut line: Local<f64>,
    mut to_server: MessageWriter<ToServer>,
) {
    let step = SNOW_LINE_SPEED * time.delta_secs_f64();
    match *cycle {
        TemperatureCycle::Off => {
            *line = weather.snow_line;
            return;
        }
        TemperatureCycle::Cooling => {
            *line = (*line - step).max(0.0);
            if *line == 0.0 {
                *cycle = TemperatureCycle::Warming;
            }
        }
        TemperatureCycle::Warming => {
            *line = (*line + step).min(NORMAL_SNOW_LINE);
            if *line == NORMAL_SNOW_LINE {
                *cycle = TemperatureCycle::Off;
            }
        }
    }
    let whole = line.round();
    if whole != weather.snow_line {
        to_server.write(ToServer(ClientMessage::DevSnowLine(whole as u32)));
    }
}
