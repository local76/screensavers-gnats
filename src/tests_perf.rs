use std::time::{Duration, Instant};
use crate::gnats::Gnats;
use crate::runner::core::screensaver::Screensaver;
use crate::runner::core::TerminalCell;

#[test]
fn test_performance_gnats() {
    let mut gnats = Gnats::new();
    // Prevent slow system info calls by setting the refresh timer to a large negative value
    gnats.sys_refresh_timer = -1000.0;

    let cols = 80;
    let rows = 24;
    let mut grid = vec![TerminalCell::default(); cols * rows];
    let dt = Duration::from_millis(16);

    let start = Instant::now();

    for _ in 0..100 {
        gnats.update(dt, cols, rows);
        gnats.draw(&mut grid, cols, rows);
    }

    let elapsed = start.elapsed();
    println!("Gnats performance test (100 frames) completed in: {:?}", elapsed);
    
    // Assert it completes within a reasonable budget (e.g. 1500ms)
    assert!(elapsed < Duration::from_millis(1500), "Performance test exceeded budget: {:?}", elapsed);
}
