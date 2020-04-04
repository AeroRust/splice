use std::cmp;
use std::env;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::io::SeekFrom;
use std::io::{BufRead, BufReader};


/* ASSEMBLY OPCODES*/
const OP_NOP:i8 = 0x00; // No action
const OP_MOV:i8 = 0x01; // OPCODE|  PREFIX|  REG_A|   DEST|
const OP_LEA:i8 = 0x02; // OPCODE|  REG_ID|TASK_ID|ADDRESS| (REG_ID can be float or int)
const OP_CMP:i8 = 0x03; // OPCODE|OPERATOR|  REG_A|  REG_B| (compare register values)
                        // OPCODE|OPERATOR|TASK_ID| REG_ID| (check task execution status)
const OP_SET:i8 = 0x04; // OPCODE| INST_ID|  PARAM| REG_ID| (sets instrument parameter to register)
                        // OPCODE|INST_REG|  CONST| REG_ID| (sets a register to fixed const)
const OP_GET:i8 = 0x05; // OPCODE| INST_ID|  PARAM| REG_ID| (gets instrument parameter from register)
const OP_ACT:i8 = 0x06; // OPCODE| INST_ID| ACTION| REG_ID|
const OP_HLT:i8 = 0x07; // Stop execution
const OP_STR:i8 = 0x08; // OPCODE|  PREFIX| UNUSED| REG_ID| provides some measure of return data? write to file?
const OP_FMA:i8 = 0x09; // OPCODE|   REG_A|  REG_B|  REG_C| add and multiply: REG_C = (REG_C+REG_B)*REG_A
const OP_FSD:i8 = 0x0A; // OPCODE|   REG_A|  REG_B|  REG_C| sub and divide: REG_C = (REG_C-REG_B)/REG_A
const OP_SIN:i8 = 0x0B; // OPCODE|  PREFIX|  REG_A|  REG_B| sine:   REG_B = sin(REG_A) ..and arcsin
const OP_COS:i8 = 0x0C; // OPCODE|  PREFIX|  REG_A|  REG_B| cosine: REG_B = cos(REG_A) ...and arccos
const OP_TAN:i8 = 0x0D; // OPCODE|  PREFIX|  REG_A|  REG_B| tan/atan: REG_B = tan(REG_A)
const OP_POW:i8 = 0x0E; // OPCODE|  PREFIX|  REG_A|  REG_B| power: log and roots can be done as well
const OP_NOR:i8 = 0x0F; // OPCODE|   REG_A|  REG_B|  REG_C| REG_C = REG_A NOR NEG_B (or NAND if neccessary)

fn process_line(p_line: String, p_mode:i32)->i32
{
    //for each code line
    println!("{}", p_line);

    //decode opcode
    //decode operands
    //for each data line
    //decode value
    return p_mode;
}


fn read_source_file(filename: String)
{
    //switch mode
    //for each line, process line
    println!("Reading from file {}\n",filename);
    //let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let mut mode:i32 = 0;
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    // Read the file line by line using the lines() iterator from std::io::BufRead.
    for (index, line) in reader.lines().enumerate() {
        let line = line.unwrap(); // Ignore errors.
        // Show the line and its number.
        mode = process_line(line, mode);
    }
}

fn main()
{
    let args: Vec<String> = env::args().collect();
    match args.len()
    {
        1=>{
            println!("Show help");
        },
        3=>{
            let cmd = &args[1];
            let arg = &args[2];
            if cmd=="-s"
            {
                let filepath: String = match arg.parse()
                {
                    Ok(path) => {
                        path
                    },
                    Err(_) => {
                        return;
                    }
                };
                println!("Translating assembly to opcodes!\n");
                read_source_file(filepath);
            }
        },
        _ => {
            println!("Show help");
        }

    }
}
