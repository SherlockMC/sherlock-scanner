use rand::Rng;
use std::net::Ipv4Addr;
use std::process::{Command, Stdio};
use std::fs;
use serde_json::Value;
use mongodb::{Client, options::ClientOptions, bson::{self, doc}};
use tokio::runtime::Runtime;

fn main() {

    println!("\r   _____ _               _            _    __  __  _____ \r\n  / ____| |             | |          | |  |  \\/  |/ ____|\r\n | (___ | |__   ___ _ __| | ___   ___| | _| \\  / | |     \r\n  \\___ \\| \'_ \\ / _ \\ \'__| |/ _ \\ / __| |/ / |\\/| | |     \r\n  ____) | | | |  __/ |  | | (_) | (__|   <| |  | | |____ \r\n |_____/|_| |_|\\___|_|  |_|\\___/ \\___|_|\\_\\_|  |_|\\_____|\n");

    loop {

        if let Err(e) = scan() {
            eprintln!("Error: {}", e);
        }

    }

}

fn scan() -> std::io::Result<()> {

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

    let start_ip = Ipv4Addr::from(ip);
    let end_ip = Ipv4Addr::from(ip + 1000000);

    let ranges = format!("{}-{}", start_ip, end_ip);
    let rangesformat = format!("{} - {}", start_ip, end_ip);

    let exclusions_path = "exclusions.txt";

    println!("Starting scan on {}.", rangesformat);

    let output = Command::new("masscan")
        .args(&[
            "-p",
            "25565",
            "--rate",
            "10000",
            &ranges,
            "-oJ",
            "out.json",
            "--excludeFile",
            &exclusions_path
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to execute masscan");

    if output.status.success() {

        println!("Completed Scan {}. Uploading Results to DB.", &rangesformat);

        if let Err(e) = mongoupload() {
            eprintln!("Error: {}", e);
        }

    } else {

        println!(
            "Scan failed with error code: {}",
            output.status.code().unwrap_or(-1)
        );
    }

    Ok(())

}

fn mongoupload() -> mongodb::error::Result<()> {
    let uri = "mongodb+srv://zarkozy:ImwSfgViwh6lBLxp@sherlock.yfrph1d.mongodb.net/?retryWrites=true&w=majority";
    let client_options = ClientOptions::parse(uri)?;
    let client = Client::with_options(client_options)?;

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let _ = client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await?;

        let collection = client.database("Sherlock").collection("ScanOutput");

        let file_content = fs::read_to_string("out.json").expect("Failed to read file");

        if file_content.is_empty() {
            println!("The file is empty, this may be an error!");
            println!("Restarting Scan.");
            fs::remove_file("out.json").expect("Failed to delete file");
            return Ok(());
        }

        let json_data: Vec<Value> = serde_json::from_str(&file_content).expect("Failed to parse JSON");

        for data in json_data {
            let ip = match data["ip"].as_str() {
                Some(ip) => ip,
                None => {
                    println!("Invalid JSON format: missing 'ip' field");
                    continue;
                }
            };

            let ports = match data["ports"].as_array() {
                Some(ports) => ports,
                None => {
                    println!("Invalid JSON format: missing 'ports' field for IP: {}", ip);
                    continue;
                }
            };

            let mut port_docs = Vec::new();
            for port in ports {
                let port_doc = match bson::to_bson(&port) {
                    Ok(doc) => doc,
                    Err(e) => {
                        println!("Failed to convert port data to Bson for IP: {}. Error: {}", ip, e);
                        continue;
                    }
                };
                port_docs.push(port_doc);
            }

            let document = doc! { "ip": ip, "ports": port_docs };
            match collection.insert_one(document, None).await {
                Ok(_) => (),
                Err(e) => println!("Failed to add IP: {}. Error: {}", ip, e),
            };
        }

        println!("Upload complete. Restarting Scan.");

        fs::remove_file("out.json").expect("Failed to delete file");

        Ok(())
    })
}