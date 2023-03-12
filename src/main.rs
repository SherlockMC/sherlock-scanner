use rand::Rng;
use std::process::Command;
use std::net::Ipv4Addr;
use std::fs::File;
use std::io::Write;

fn main() {
    let mut rng = rand::thread_rng();
    let mut ip: u32;

    loop {
        let octet1 = rng.gen_range(1..224);
        if octet1 == 10 || octet1 == 127 || octet1 == 169 || octet1 == 192 {
            continue;
        }
        ip = (octet1 as u32) << 24;
        ip |= rng.gen_range(0..256) << 16;
        ip |= rng.gen_range(0..256) << 8;
        ip |= 0x000000FF;
        break;
    }

    let mut ips: Vec<Ipv4Addr> = Vec::with_capacity(16_000_000);

    for i in 0..16_000_000 {
        let new_ip = ip + i;
        ips.push(Ipv4Addr::from(new_ip));
    }

    let chunk_size = 100;
    let num_chunks = ips.len() / chunk_size;

    for i in 0..num_chunks {
        let chunk_start = i * chunk_size;
        let chunk_end = chunk_start + chunk_size;
        let chunk = &ips[chunk_start..chunk_end];

        let ranges = chunk.iter().map(|ip| ip.to_string()).collect::<Vec<String>>().join(",");

        let output = Command::new("masscan")
        .args(&[
            "-p", "25565",
            "--rate", "10000",
            &ranges,
        ])
        .output()
        .expect("Failed to execute masscan");

        let filename = format!("scan_chunk_{}.txt", i);

        let mut file = File::create(filename)
            .expect("Failed to create file");

        if output.status.success() {
            println!("Scan completed successfully");
            let stdout = String::from_utf8(output.stderr).unwrap();
            println!("Output: \n{}", stdout);
            file.write_all(stdout.as_bytes())
                .expect("Failed to write output to file");
        } else {
            println!("Scan failed with error code: {}", output.status.code().unwrap_or(-1));
        }
    }
}
