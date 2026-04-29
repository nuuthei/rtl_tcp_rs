use std::{io::Read, net::TcpStream};
use std::{env, f64};
use std::fs::File;
use std::io::Write;
use dotenv::dotenv;
use std::collections::VecDeque;

fn avg_filter(s: f64, state: &mut VecDeque<f64>) -> f64 {
    if state.len() < 20 {
        state.push_back(s);
    }
    else {
        state.pop_front();
        state.push_back(s);
    }

    let sum: f64 = state.iter().copied().sum();
    sum / state.len() as f64
}

fn design_lowpass(num_taps: usize, cutoff: f64) -> Vec<f64> {
    let mut h = vec![0.0; num_taps];
    let m = (num_taps as f64 - 1.0) / 2.0;

    for n in 0..num_taps {
        let x = n as f64 - m;

        // sinc
        let sinc = if x == 0.0 {
            2.0 * cutoff
        } else {
            (2.0 * std::f64::consts::PI * cutoff * x).sin()
                / (std::f64::consts::PI * x)
        };

        // Hamming window
        let w = 0.54 - 0.46 * (2.0 * std::f64::consts::PI * n as f64 / (num_taps as f64 - 1.0)).cos();

        h[n] = sinc * w;
    }

    // normalize
    let sum: f64 = h.iter().sum();
    for v in &mut h {
        *v /= sum;
    }

    h
}

fn fir_filter(x: f64, state: &mut [f64], taps: &[f64]) -> f64 {
    state.rotate_right(1);
    state[0] = x;

    taps.iter()
        .zip(state.iter())
        .map(|(h, x)| h * x)
        .sum()
}

fn handle_stream_fm_demod(mut stream: TcpStream) -> std::io::Result<()> {
    let mut header = [0u8; 12];
    stream.read_exact(&mut header)?;

    let fir = design_lowpass(63, 15_000.0 / 240_000.0);
    let mut fir_state: Vec<f64> = vec![0.0; 63];

    let mut buffer = [0u8; 16384];
    let mut out_buffer = [0f64; 3200];
    let mut file = File::create("out.raw")?;
    let decim = 5;

    let mut window: VecDeque<f64> = VecDeque::new();

    let mut i_prev: f64 = 0.0;
    let mut q_prev: f64 = 0.0;
    
    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        let mut j = 0;

        for i in (0..n).step_by(2) {
            let i_sample = buffer[i] as f64 - 128.0;
            let q_sample = buffer[i + 1] as f64 - 128.0;

            let mag = (i_sample*i_sample + q_sample*q_sample).sqrt();
            let i_new = i_sample / mag;
            let q_new = q_sample / mag;

            let diff = q_new * i_prev - q_prev * i_new;
            i_prev = i_sample;
            q_prev = q_sample;

            let diff_filtered = avg_filter(diff, &mut window);

            let filtered = fir_filter(diff_filtered, &mut fir_state, &fir);

            out_buffer[j] = filtered;

            if (i / 2) % decim == 0 {
                out_buffer[j] = filtered;
                j += 1;
            }
            
        }
        
        for k in 1..j {
            let sample_i16 = (out_buffer[k] * 32767.0)
            .clamp(-32768.0, 32767.0) as i16;

            file.write_all(&sample_i16.to_le_bytes())?;
        }
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    println!("Hello, world!");

    let recv_adr = env::var("RECV_ADR").expect("RECV_ADR not set!");

    let stream = TcpStream::connect(recv_adr)?;

    handle_stream_fm_demod(stream)?;

    Ok(())
}
