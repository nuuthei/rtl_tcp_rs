use std::{io::Read, net::TcpStream};
use std::{env, f64};
use std::fs::File;
use std::io::Write;
use dotenv::dotenv;

fn handle_stream_fm_demod(mut stream: TcpStream) -> std::io::Result<()> {
    let mut header = [0u8; 12];
    stream.read_exact(&mut header)?;

    let mut buffer = [0u8; 16384];
    let mut out_buffer = [0f64; 8192];
    let mut file = File::create("out.raw")?;
    let decim = 10;

    let mut i_prev: f64 = 0.0;
    let mut q_prev: f64 = 0.0;
    
    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        let n = n & !1;

        for i in (0..n).step_by(2) {
            let i_sample = buffer[i] as f64 - 128.0;
            let q_sample = buffer[i + 1] as f64 - 128.0;

            let d = q_sample.atan2(i_sample) - q_prev.atan2(i_prev);
            i_prev = i_sample;
            q_prev = q_sample;

            if (i / 2) % decim == 0 {
                out_buffer[i / 2] = d;
            }
            
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
