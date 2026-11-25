//! Serial write-path latency probe.
//!
//! Usage: cargo run -p vterm-server --example serial_write_probe -- [PORT] [BAUD]
//!
//! Measures, on the real OS/driver path:
//!   1. cold first write_all (thread + driver warmup)
//!   2. steady-state write_all of 1 byte
//!   3. flush (Windows: FlushFileBuffers — drains TX at baud rate)
//!   4. write_all + flush combined (the old per-keystroke path)

use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port = args.get(1).cloned().unwrap_or_else(|| "COM4".to_string());
    let baud: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(115200);

    let port = serialport::new(&port, baud)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .timeout(std::time::Duration::from_millis(10))
        .open();

    let mut port = match port {
        Ok(p) => p,
        Err(e) => {
            eprintln!("failed to open port: {e}");
            std::process::exit(1);
        }
    };
    println!(
        "opened {port_name} @ {baud} baud",
        port_name = args.get(1).cloned().unwrap_or_else(|| "COM4".to_string())
    );

    let payload = [b'a'];

    // 1. cold write
    let t = Instant::now();
    let _ = port.write_all(&payload);
    println!(
        "cold write_all(1B):   {:>8.3} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );

    // 2. steady-state write_all
    let mut max_ms = 0.0f64;
    let mut sum = 0.0f64;
    for _ in 0..20 {
        let t = Instant::now();
        let _ = port.write_all(&payload);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        sum += ms;
        max_ms = max_ms.max(ms);
    }
    println!(
        "write_all(1B) x20:    avg {:>7.3} ms  max {:>7.3} ms",
        sum / 20.0,
        max_ms
    );

    // 3. write + flush (old per-keystroke path)
    let mut max_ms = 0.0f64;
    let mut sum = 0.0f64;
    for i in 0..20 {
        let t = Instant::now();
        let _ = port.write_all(&payload);
        let w = t.elapsed().as_secs_f64() * 1000.0;
        let t2 = Instant::now();
        let _ = port.flush();
        let f = t2.elapsed().as_secs_f64() * 1000.0;
        let total = t.elapsed().as_secs_f64() * 1000.0;
        sum += total;
        max_ms = max_ms.max(total);
        if i < 5 {
            println!(
                "  write+flush #{i}:    write {:>7.3} ms  flush {:>7.3} ms  total {:>7.3} ms",
                w, f, total
            );
        }
    }
    println!(
        "write_all+flush x20:  avg {:>7.3} ms  max {:>7.3} ms",
        sum / 20.0,
        max_ms
    );
}
