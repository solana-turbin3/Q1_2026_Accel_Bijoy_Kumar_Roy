use crate::constants::{FIVE_PM_UTC, MARGIN, NINE_AM_UTC, SECONDS_IN_A_DAY};
pub fn is_transfer_window_open(current_timestamp: i64) -> bool {
    // Get seconds passed since the start of the current UTC day
    let seconds_since_midnight = current_timestamp % SECONDS_IN_A_DAY;

    seconds_since_midnight >= NINE_AM_UTC && seconds_since_midnight <= FIVE_PM_UTC
}

pub fn is_near_boundary(unix_timestamp: i64) -> bool {
    let seconds_since_midnight = unix_timestamp % 86400;

    // Checks if we are in the first 1 hour of opening OR first 1 hour of closing
    let opening_bell =
        seconds_since_midnight >= NINE_AM_UTC && seconds_since_midnight < (NINE_AM_UTC + MARGIN);

    let closing_bell =
        seconds_since_midnight >= (FIVE_PM_UTC - MARGIN) && seconds_since_midnight < FIVE_PM_UTC;

    opening_bell || closing_bell
}
