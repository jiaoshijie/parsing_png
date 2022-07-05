use std::cmp::Ordering;
use std::env;
use std::fs::File;
use std::io::prelude::*;
#[allow(unused_imports)]
use std::io::SeekFrom;
use std::process;

use memx::memcmp;

// PNG Structure
// http://libpng.org/pub/png/spec/1.2/PNG-Structure.html

const PNG_SIG_LEN: usize = 8;
const PNG_SIG: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap();
    let input_file_path = match args.next() {
        Some(v) => v,
        None => {
            eprintln!("Usage: {} <input.png> <output.png>", program);
            eprintln!("ERROR: no input file is provided!!!");
            process::exit(1);
        }
    };

    let output_file_path = match args.next() {
        Some(v) => v,
        None => {
            eprintln!("Usage: {} <input.png> <output.png>", program);
            eprintln!("ERROR: no output file is provided!!!");
            process::exit(1);
        }
    };

    let mut input_file = match File::options().read(true).open(&input_file_path) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("ERROR: can not open file {}: {}", input_file_path, err);
            process::exit(1);
        }
    };

    let mut output_file = match File::options()
        .write(true)
        .create(true)
        .open(&output_file_path)
    {
        Ok(v) => v,
        Err(err) => {
            eprintln!("ERROR: can not open file {}: {}", output_file_path, err);
            process::exit(1);
        }
    };

    let mut sig = vec![0u8; PNG_SIG_LEN];

    input_file.read_exact(&mut sig).unwrap_or_else(|err| {
        eprintln!(
            "ERROR: read file {} input png Header failed: {}",
            input_file_path, err
        );
        process::exit(1);
    });

    if memcmp(&PNG_SIG, &sig) != Ordering::Equal {
        eprintln!("ERROR: The file {} png Header is wrong", input_file_path);
        process::exit(1);
    }

    output_file.write_all(&PNG_SIG).unwrap();

    // NOTICE: inject some something to png image file.
    let inject_data = String::from("shijie");
    let inject_len = convert_u32_2_u8(inject_data.len() as u32);
    output_file.write_all(&inject_len).unwrap();
    output_file.write_all("jiAO".as_bytes()).unwrap();
    output_file.write_all(inject_data.as_bytes()).unwrap();
    // TODO: calc the correct inject chunk crc
    let inject_crc = convert_u32_2_u8(0u32);
    output_file.write_all(&inject_crc).unwrap();

    loop {
        let mut chunk_size = vec![0; 4];
        input_file.read_exact(&mut chunk_size).unwrap();
        output_file.write_all(&chunk_size).unwrap();
        let chunk_size = convert_u8_2_u32(chunk_size);

        let mut chunk_type = vec![0; 4];
        input_file.read_exact(&mut chunk_type).unwrap();
        output_file.write_all(&chunk_type).unwrap();
        let chunk_type = String::from_utf8(chunk_type).unwrap();

        // NOTICE: read input file chunk data to output file chunk data
        let mut n = chunk_size as usize;
        while n > 0 {
            let mut m = n;
            if m > 1024 {
                m = 1024;
            }
            n -= m;
            let mut chunk_buf = vec![0; m];
            input_file.read(&mut chunk_buf).unwrap();
            output_file.write(&chunk_buf).unwrap();
        }

        // NOTICE: move file pointer to chunk crc.
        // input_file
        //     .seek(SeekFrom::Current(chunk_size as i64))
        //     .unwrap();

        let mut chunk_crc = vec![0; 4];
        input_file.read_exact(&mut chunk_crc).unwrap();
        output_file.write_all(&chunk_crc).unwrap();
        reverse_bytes(&mut chunk_crc);
        let chunk_crc = convert_u8_2_u32(chunk_crc);

        println!("chunk size: {}", chunk_size);
        println!("chunk type: {}", chunk_type);
        println!("{:#08X?}", chunk_crc);
        if chunk_type == String::from("IEND") {
            break;
        }
        println!("--------------------------");
    }
}

fn convert_u8_2_u32(bytes: Vec<u8>) -> u32 {
    let mut ret: u32 = 0u32;
    let len = bytes.len();
    for i in 0..len {
        let t: u32 = bytes[i] as u32;
        ret += t << (8 * (len - i - 1));
    }
    ret
}

fn convert_u32_2_u8(size: u32) -> Vec<u8> {
    let mut ret = Vec::new();
    let mut mask: u32 = 0xFF000000u32;

    for i in 0..4 {
        let t = (size & mask) >> (8 * (3 - i));
        ret.push(t as u8);
        mask >>= 8;
    }
    ret
}

fn reverse_bytes(bytes: &mut Vec<u8>) {
    let len = bytes.len();
    for i in 0..len / 2 {
        let t = bytes[i];
        bytes[i] = bytes[len - i - 1];
        bytes[len - i - 1] = t;
    }
}
