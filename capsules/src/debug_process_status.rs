//! Periodic process status display.
//!
//! This capsule periodically prints the status of all of the apps on the board.
//!
//! Usage
//! -----
//!
//! ```
//! let debug_process_status_virtual_alarm = static_init!(
//!     VirtualMuxAlarm<'static, sam4l::ast::Ast>,
//!     VirtualMuxAlarm::new(mux_alarm)
//! );
//! let introspection = static_init!(
//!     kernel::introspection::Introspection,
//!     kernel::introspection::Introspection::new(board_kernel)
//! );
//! let debug_process_status = static_init!(
//!     capsules::debug_process_status::DebugProcessStatus<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
//!     capsules::debug_process_status::DebugProcessStatus::new(debug_process_status_virtual_alarm, introspection)
//! );
//! debug_process_status_virtual_alarm.set_client(debug_process_status);
//! ```

use kernel::common::cells::OptionalCell;
use kernel::hil;
use kernel::hil::time::Frequency;
use kernel::introspection::Introspection;

pub struct DebugProcessStatus<'a, A: hil::time::Alarm + 'a> {
    alarm: &'a A,
    inspector: &'a Introspection,
    interval: OptionalCell<u32>,
}

impl<'a, A: hil::time::Alarm + 'a> DebugProcessStatus<'a, A> {
    pub fn new(alarm: &'a A, inspector: &'a Introspection) -> DebugProcessStatus<'a, A> {
        DebugProcessStatus {
            alarm: alarm,
            inspector: inspector,
            interval: OptionalCell::empty(),
        }
    }

    /// Enable the debugging display and have it start printing the status of
    /// each process.
    pub fn start(&self, interval_ms: usize) {
        let interval = (interval_ms as u32) * <A::Frequency>::frequency() / 1000;
        self.interval.set(interval);
        let tics = self.alarm.now().wrapping_add(interval);
        self.alarm.set_alarm(tics);
    }

    /// Stop the periodic debugging display.
    pub fn stop(&self) {
        self.interval.clear();
    }
}

impl<'a, A: hil::time::Alarm + 'a> hil::time::Client for DebugProcessStatus<'a, A> {
    fn fired(&self) {
        debug!("##### APP INFORMATION #####");

        let process_count = self.inspector.number_loaded_processes();
        debug!("  Number of processes: {}\n", process_count);

        self.inspector.each_app(|appid| {
            debug!("  app: {}", self.inspector.process_name(appid));
            debug!(
                "    # syscalls:              {}",
                self.inspector.number_app_syscalls(appid)
            );
            debug!(
                "    # dropped callbacks:     {}",
                self.inspector.number_app_dropped_callbacks(appid)
            );
            debug!(
                "    # restarts:              {}",
                self.inspector.number_app_restarts(appid)
            );
            debug!(
                "    # timeslice expirations: {}\n",
                self.inspector.number_app_timeslice_expirations(appid)
            );
        });

        // Restart the timer unless `stop()` was called.
        self.interval.map(|interval| {
            let tics = self.alarm.now().wrapping_add(*interval);
            self.alarm.set_alarm(tics);
        });
    }
}
