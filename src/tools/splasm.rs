use std::cmp;
use std::env;
use std::fmt;
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

const PRE_MOV_REG:i8 = 0x01;
const PRE_MOV_RAM:i8 = 0x02;
const PRE_MOV_IND:i8 = 0x03;

fn pack4x8to32(a:i8, b:i8, c:i8, d:i8)->i32
{
    let ax:i32 = a.into();
    let bx:i32 = b.into();
    let cx:i32 = c.into();
    let dx:i32 = d.into();
    let result:i32 = (ax<<24)|(bx<<16)|(cx<<8)|dx;
    return result;
}

fn decode_prefix(p_pref: String)->i8
{
    match p_pref.as_str()
    {
        "PRE_MOV_REG"=>{ return 0x01 }
        "PRE_MOV_RAM"=>{ return 0x02 }
        "PRE_MOV_IND"=>{ return 0x03 }
        "PRE_STR_ALU"=>{ return 0x01 }
        "PRE_STR_FPU"=>{ return 0x02 }
        "PRE_STR_BIN"=>{ return 0x03 }
        "PRE_NORMAL"=> { return 0x01 }
        "PRE_INVERT"=> { return 0x02 }
        _=>{ return -1 }
    }
}

fn decode_address(p_addr: String)->i8
{
    let result:i8 = p_addr.parse().unwrap();
    return result;
}

fn decode_register(reg_id: String)->i8
{
    match reg_id.as_str()
    {
        //ALU registers
        "IREG_A"=> { return 0x00 }
        "IREG_B"=> { return 0x01 }
        "IREG_C"=> { return 0x02 }
        "IREG_D"=> { return 0x03 }
        "IREG_E"=> { return 0x04 }
        "IREG_F"=> { return 0x05 }
        "IREG_G"=> { return 0x06 }
        "IREG_H"=> { return 0x07 }
        "IREG_I"=> { return 0x08 }
        "IREG_J"=> { return 0x09 }
        "IREG_K"=> { return 0x0A }
        "IREG_L"=> { return 0x0B }
        "IREG_M"=> { return 0x0C }
        "IREG_N"=> { return 0x0D }
        "IREG_P"=> { return 0x0E }
        "IREG_U"=> { return 0x0F }
        // FPU registers
        "FREG_A"=> { return 0x10 }
        "FREG_B"=> { return 0x11 }
        "FREG_C"=> { return 0x12 }
        "FREG_D"=> { return 0x13 }
        "FREG_E"=> { return 0x14 }
        "FREG_F"=> { return 0x15 }
        "FREG_G"=> { return 0x16 }
        "FREG_H"=> { return 0x17 }
        "FREG_I"=> { return 0x18 }
        "FREG_J"=> { return 0x19 }
        "FREG_K"=> { return 0x1A }
        "FREG_L"=> { return 0x1B }
        "FREG_M"=> { return 0x1C }
        "FREG_N"=> { return 0x1D }
        "FREG_P"=> { return 0x1E }
        "FREG_U"=> { return 0x1F }
        _=>{ return -1 }
    }
}

fn process_line(p_line: String, mut p_mode:i32)->i32
{
    //for each code line
    //println!("{}", p_line);
    let mut line_values:Vec<&str> = p_line.split(",").collect();
    match p_mode
    {
        0=>{
            let group_id:i8 = line_values[0].to_string().parse().unwrap();
            let task_id:i8 = line_values[1].to_string().parse().unwrap();
            let freq:i8 = line_values[2].to_string().parse().unwrap();
            let length:i8 = line_values[3].to_string().parse().unwrap();
            let bytecode:i32 = pack4x8to32(group_id, task_id, freq, length);
            print!("{:x}",bytecode);
            p_mode = p_mode + 1;
        }
        1=>{
            let opcode:String = line_values[0].to_string();
            match opcode.as_str()
            {
                "OP_NOP"=>{
                    let bytecode:i32 = pack4x8to32(OP_NOP, OP_NOP, OP_NOP, OP_NOP);
                    print!("{:x}",bytecode);
                }
                "OP_HLT"=>{
                    let bytecode:i32 = pack4x8to32(OP_HLT, OP_NOP, OP_NOP, OP_NOP);
                    print!("{:x}",bytecode);
                }
                "OP_LEA"=>{
                    let op_a:i8 = decode_register(line_values[1].trim().to_string());
                    let op_b:i8 = decode_address(line_values[2].trim().to_string());
                    let op_c:i8 = decode_address(line_values[3].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_LEA, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_MOV"=>{
                    let op_a:i8 = decode_prefix(line_values[1].trim().to_string());
                    let op_b:i8 = decode_register(line_values[2].trim().to_string());
                    let mut op_c:i8=0;
                    if (op_a == PRE_MOV_REG)
                    {
                        op_c = decode_register(line_values[3].trim().to_string());
                    }
                    if (op_a == PRE_MOV_RAM)
                    {
                        op_c= decode_address(line_values[3].trim().to_string());
                    }
                    let bytecode:i32 = pack4x8to32(OP_MOV, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                _=>{
                    //println!("Internal error: unrecognized opcode");
                }
            }

        }
        _=>{
            println!("Internal error: unrecognized read mode");
        }
    }
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
    let mut first:bool = true;
    for (index, line) in reader.lines().enumerate() {
        let line = line.unwrap(); // Ignore errors.
        // Show the line and its number.
        if (!first) {
            print!(",");
        }
        mode = process_line(line, mode);
        first = false;

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
                //println!("Translating assembly to opcodes!\n");
                read_source_file(filepath);
            }
        },
        _ => {
            println!("Show help");
        }

    }
}
