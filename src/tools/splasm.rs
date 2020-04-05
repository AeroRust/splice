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
use std::mem;



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

//cmp operators
const TSX_EQ:i8 = 0x0D; //task result is equal ...
const TSX_NE:i8 = 0x0E; //task result is not equal to ...

//prefixes
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

fn decode_operator(p_oper: String)->i8
{
  match p_oper.as_str()
  {
     "ALU_EQ" => { return 0x01 }
     "ALU_NE" => { return 0x02 }
     "ALU_GT" => { return 0x03 }
     "ALU_LT" => { return 0x04 }
     "ALU_GE" => { return 0x05 }
     "ALU_LE" => { return 0x06 } //lesser or equal
     "FPU_EQ" => { return 0x07 } //same but for FPU
     "FPU_NE" => { return 0x08 }
     "FPU_GT" => { return 0x09 }
     "FPU_LT" => { return 0x0A }
     "TSX_EQ" => { return 0x0D } //task result is equal ...
     "TSX_NE" => { return 0x0E } //task result is not equal to ...
     _=> { return -1 }
  }
}

fn decode_action(p_action_id: String)->i8
{
  match p_action_id.as_str()
  {
      "A_IMG_DO_JPG" => { return 0x07 }
      "A_IMG_DO_RAW" => { return 0x08 }
      "A_IMG_DO_BMP" => { return 0x09 }
      "A_IMG_DO_PNG" => { return 0x0A }
      "A_ADC_NADIR" =>  { return 0x05 }
      "A_ADC_TOSUN" => { return 0x06 }
      "A_ADC_BDOTT" => { return 0x07 }
      "A_ADC_TRACK" => { return 0x08 }
      "A_ADC_UNSET" => { return 0x09 }
       _=> { return -1 }
  }
}

fn decode_instrument(p_inst: String)->i8
{
    match p_inst.as_str()
    {
        "INST_ADC" => { return 0x01 }
        "INST_GPS" => { return 0x02 }
        "INST_IMG" => { return 0x03 }
        "INST_FPU" => { return 0x04 } //load a constant to FPU register
        "INST_SDR" => { return 0x05 } //not supported yet
        "INST_NMF" => { return 0x06 } //set or get NMF-related parameter
        "INST_VXM" => { return 0x07 } //set or get internal VM parameter
      _=> { return -1 }
    }
}

fn decode_parameter(p_param: String)->i8
{
  match p_param.as_str()
  {
      "P_ADC_MODE" => { return 0x01 }
      "P_ADC_MAGX" => { return 0x02 }
      "P_ADC_MAGY" => { return 0x03 }
      "P_ADC_MAGZ" => { return 0x04 }
      "P_ADC_SUNX" => { return 0x05 }
      "P_ADC_SUNY" => { return 0x06 }
      "P_ADC_SUNZ" => { return 0x07 }
      "P_ADC_ANGX" => { return 0x08 }
      "P_ADC_ANGY" => { return 0x09 }
      "P_ADC_ANGZ" => { return 0x0A }
      "P_ADC_QTNA" => { return 0x0B }
      "P_ADC_QTNB" => { return 0x0C }
      "P_ADC_QTNC" => { return 0x0D }
      "P_ADC_QTND" => { return 0x0E }
      "P_ADC_MTQX" => { return 0x0F }
      "P_ADC_MTQY" => { return 0x10 }
      "P_ADC_MTQZ" => { return 0x11 }
      "P_IMG_GAIN_R"=> { return 0x01 }
      "P_IMG_GAIN_G"=> { return 0x02 }
      "P_IMG_GAIN_B"=> { return 0x03 }
      "P_IMG_EXPOSE"=> { return 0x04 }
      "P_IMG_STATUS"=> { return 0x05 }  //not to be used?
      "P_IMG_NUMBER"=> { return 0x06 }
      "P_GPS_LATT" => { return 0x01 }
      "P_GPS_LONG" => { return 0x02 }
      "P_GPS_ALTT" => { return 0x03 }
      "P_GPS_TIME" => { return 0x04 }
      "P_NMF_TIME" => { return 0x01 }
      "P_VXM_TIME" => { return 0x01 }
      "P_VXM_PRSN" => { return 0x02 }
      "P_VXM_TLSC" => { return 0x03 }
      "P_VXM_DBUG" => { return 0x04 }
      "P_FPU_NIL" => { return 0x00 }
      "P_FPU_ONE" => { return 0x01 }
      "P_FPU_EXP" => { return 0x02 }
      "P_FPU_PIE" => { return 0x03 }
      _=> { return -1 }
  }
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
            let group_id:i8 = line_values[0].trim().to_string().parse().unwrap();
            let task_id:i8 = line_values[1].trim().to_string().parse().unwrap();
            let freq:i8 = line_values[2].trim().to_string().parse().unwrap();
            let length:i8 = line_values[3].trim().to_string().parse().unwrap();
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
                    p_mode = p_mode + 1;
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
                "OP_CMP"=>{
                    let op_a:i8 = decode_operator(line_values[1].trim().to_string());
                    let op_c:i8 = decode_register(line_values[3].trim().to_string());
                    let mut op_b:i8=0;
                    if ((op_a == TSX_EQ) | (op_a == TSX_NE))
                    {
                        op_b = decode_address(line_values[2].trim().to_string());
                    }
                    else
                    {
                        op_b = decode_register(line_values[2].trim().to_string());
                    }
                    let bytecode:i32 = pack4x8to32(OP_CMP, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_GET"=>{
                    let op_a:i8 = decode_instrument(line_values[1].trim().to_string());
                    let op_b:i8 = decode_parameter(line_values[2].trim().to_string());
                    let op_c:i8 = decode_register(line_values[3].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_GET, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_SET"=>{
                    let op_a:i8 = decode_instrument(line_values[1].trim().to_string());
                    let op_b:i8 = decode_parameter(line_values[2].trim().to_string());
                    let op_c:i8 = decode_register(line_values[3].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_SET, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_ACT"=>{
                    let op_a:i8 = decode_instrument(line_values[1].trim().to_string());
                    let op_b:i8 = decode_action(line_values[2].trim().to_string());
                    let op_c:i8 = decode_register(line_values[3].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_ACT, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_STR"=>{
                    let op_a:i8 = decode_prefix(line_values[1].trim().to_string());
                    let op_c:i8 = decode_register(line_values[2].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_STR, op_a, OP_NOP, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_FMA"=>{
                    let op_a:i8 = decode_register(line_values[1].trim().to_string());
                    let op_b:i8 = decode_register(line_values[2].trim().to_string());
                    let op_c:i8 = decode_register(line_values[3].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_FMA, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_FSD"=>{
                    let op_a:i8 = decode_register(line_values[1].trim().to_string());
                    let op_b:i8 = decode_register(line_values[2].trim().to_string());
                    let op_c:i8 = decode_register(line_values[3].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_FSD, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_SIN"=>{
                    let op_a:i8 = decode_prefix(line_values[1].trim().to_string());
                    let op_b:i8 = decode_register(line_values[2].trim().to_string());
                    let op_c:i8 = decode_register(line_values[3].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_SIN, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_COS"=>{
                    let op_a:i8 = decode_prefix(line_values[1].trim().to_string());
                    let op_b:i8 = decode_register(line_values[2].trim().to_string());
                    let op_c:i8 = decode_register(line_values[3].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_COS, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_TAN"=>{
                    let op_a:i8 = decode_prefix(line_values[1].trim().to_string());
                    let op_b:i8 = decode_register(line_values[2].trim().to_string());
                    let op_c:i8 = decode_register(line_values[3].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_TAN, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_POW"=>{
                    let op_a:i8 = decode_prefix(line_values[1].trim().to_string());
                    let op_b:i8 = decode_register(line_values[2].trim().to_string());
                    let op_c:i8 = decode_register(line_values[3].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_POW, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                "OP_NOR"=>{
                    let op_a:i8 = decode_register(line_values[1].trim().to_string());
                    let op_b:i8 = decode_register(line_values[2].trim().to_string());
                    let op_c:i8 = decode_register(line_values[3].trim().to_string());
                    let bytecode:i32 = pack4x8to32(OP_NOR, op_a, op_b, op_c);
                    print!("{:x}",bytecode);
                }
                _=>{
                    //println!("Internal error: unrecognized opcode");
                }
            }
        }
        2=>{
            let l_value = &p_line[0..p_line.len()-1];
            let s_value = &p_line[p_line.len()-1..p_line.len()];
            //p_line.substring(0,line.length() - 1);
            //suffix:String = p_line.charAt(line.length() - 1
            match s_value
            {
                "i"=>{
                    let bytecode:i32 = l_value.parse().unwrap();
                    print!("{:x}",bytecode);
                }
                "f"=>{
                    let fvalue:f32 = l_value.parse().unwrap();
                    let mut bytecode:i32 = 0;
                    unsafe {
                        bytecode = mem::transmute::<f32,i32>(fvalue);
                        print!("{:x}",bytecode);
                    }
                }
                _=>{
                    println!("Internal error: unrecognized data type");
                }
            }
        }
        _=>{
            println!("Internal error: unrecognized read mode");
        }
    }
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
    println!();
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
