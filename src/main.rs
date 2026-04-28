use std::{io::Read, net::TcpStream};
use std::env;
use std::fs::File;
use std::io::Write;
use dotenv::dotenv;

fn handle_stream_fm_demod(mut stream: TcpStream) -> std::io::Result<()> {
    let mut header = [0u8; 12];
    stream.read_exact(&mut header)?;

    let mut buffer = [0u8; 16384];
    let mut file = File::create("out.raw")?;

    let mut prev_i = 0.0;
    let mut prev_q = 0.0;
    let mut audio_lp = 0.0;
    let mut deemph = 0.0;

    let mut decim_counter = 0;

    let decimation = 20;

    let tau = 50e-6;
    let fs = 48_000.0;
    let alpha = 1.0 / (1.0 + fs * tau);

    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        let n = n & !1;

        for i in (0..n).step_by(2) {
            let i_sample = (buffer[i] as f64 - 128.0) / 128.0;
            let q_sample = (buffer[i + 1] as f64 - 128.0) / 128.0;

            let d = (prev_i * (q_sample - prev_q)
                   - prev_q * (i_sample - prev_i))
                   / (prev_i * prev_i + prev_q * prev_q + 1e-12);

            prev_i = i_sample;
            prev_q = q_sample;

            audio_lp += 0.05 * (d - audio_lp);

            // decimate
            decim_counter += 1;
            if decim_counter < decimation {
                continue;
            }
            decim_counter = 0;

            // de-emphasis
            deemph += alpha * (audio_lp - deemph);

            let sample = (deemph * 12000.0)
                .clamp(-32768.0, 32767.0) as i16;

            file.write_all(&sample.to_le_bytes())?;
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
