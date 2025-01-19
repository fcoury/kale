use std::{env, fs};

use kale::Keyboard;

fn write_raw_format(keyboard: &Keyboard, output_file: &str) -> std::io::Result<()> {
    let raw_format = keyboard.to_raw_format();
    fs::write(output_file, raw_format)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = env::args().nth(1).ok_or("No file name provided")?;
    let raw_data = fs::read_to_string(&file_name)
        .map_err(|e| format!("Error reading file {}: {}", file_name, e))?;

    // Trim any whitespace and handle CRLF line endings
    let raw_data = raw_data.trim().replace("\r\n", "\n");

    match Keyboard::parse(&raw_data) {
        Ok(keyboard) => {
            println!("Successfully parsed keyboard");

            // Generate output filename
            let output_file = file_name.replace(".json", "_output.json");

            // Write the raw format
            write_raw_format(&keyboard, &output_file)?;
            println!("Written to {}", output_file);
        }
        Err(e) => eprintln!("Error parsing keyboard: {}", e),
    }

    Ok(())
}
