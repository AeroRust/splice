import java.io.FileInputStream;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.io.IOException;
import java.io.FileNotFoundException;


public class SpliceTranslator
{
  /* opcode definitions */
  public static final byte OP_NOP = 0x00; // No action
  public static final byte OP_MOV = 0x01; // OPCODE|  PREFIX|  REG_A|   DEST|
  public static final byte OP_LEA = 0x02; // OPCODE|  REG_ID|TASK_ID|ADDRESS| (REG_ID can be float or int)
  public static final byte OP_CMP = 0x03; // OPCODE|OPERATOR|  REG_A|  REG_B| (compare register values)
                                          // OPCODE|OPERATOR|TASK_ID| REG_ID| (check task execution status)
  public static final byte OP_SET = 0x04; // OPCODE| INST_ID|  PARAM| REG_ID| (sets instrument parameter to register)
                                          // OPCODE|INST_REG|  CONST| REG_ID| (sets a register to fixed const)
  public static final byte OP_GET = 0x05; // OPCODE| INST_ID|  PARAM| REG_ID| (gets instrument parameter from register)
  public static final byte OP_ACT = 0x06; // OPCODE| INST_ID| ACTION| REG_ID|
  public static final byte OP_HLT = 0x07; // Stop execution
  public static final byte OP_STR = 0x08; // OPCODE|  PREFIX| UNUSED| REG_ID| provides some measure of return data? write to file?
  public static final byte OP_FMA = 0x09; // OPCODE|   REG_A|  REG_B|  REG_C| add and multiply: REG_C = (REG_C+REG_B)*REG_A
  public static final byte OP_FSD = 0x0A; // OPCODE|   REG_A|  REG_B|  REG_C| sub and divide: REG_C = (REG_C-REG_B)/REG_A
  public static final byte OP_SIN = 0x0B; // OPCODE|  PREFIX|  REG_A|  REG_B| sine:   REG_B = sin(REG_A) ..and arcsin
  public static final byte OP_COS = 0x0C; // OPCODE|  PREFIX|  REG_A|  REG_B| cosine: REG_B = cos(REG_A) ...and arccos
  public static final byte OP_TAN = 0x0D; // OPCODE|  PREFIX|  REG_A|  REG_B| tan/atan: REG_B = tan(REG_A)
  public static final byte OP_POW = 0x0E; // OPCODE|  PREFIX|  REG_A|  REG_B| power: log and roots can be done as well
  public static final byte OP_NOR = 0x0F; // OPCODE|   REG_A|  REG_B|  REG_C| REG_C = REG_A NOR NEG_B (or NAND if neccessary)
  //cmp operators
  public static final byte TSX_EQ = 0x0D; //task result is equal ...
  public static final byte TSX_NE = 0x0E; //task result is not equal to ...
  /* prefixes */
  //MOV addressing modes
  public static final byte PRE_MOV_REG = 0x01;
  public static final byte PRE_MOV_RAM = 0x02;
  public static final byte PRE_MOV_IND = 0x03;
  //VM frequency modifiers
  /*
  public static final byte FREQ_ONCE = 0x00;
  public static final byte FREQ_1MIN = 0x3C; 60
  public static final byte FREQ_HOUR = 0x77; 119
  public static final byte FREQ_TMAX = 0x7F; 127
  */

  /* byte manipulation mask */
  public static final int BYTE_MASK_GET = 0x000000FF;

  //SpliceVM code generator;
  SpliceCodeGen VMT = new SpliceCodeGen();

  /* VM definitions */
  public class SpliceCodeGen
  {
    private byte[] unpack32to4x8(int inbound)
    {
      byte[] unpacked = new byte[4];
      unpacked[0] = (byte)((inbound>>>24)&BYTE_MASK_GET); //group id | OPCODE
      unpacked[1] = (byte)((inbound>>>16)&BYTE_MASK_GET); //task id  | OPERATOR or INSTRUMENT
      unpacked[2] = (byte)((inbound>>>8)&BYTE_MASK_GET);  //frequency| REGISTER or PARAMETER
      unpacked[3] = (byte)(inbound&BYTE_MASK_GET);        //offset id| REGISTER
      return unpacked;
    }

    private int pack4x8to32(byte a, byte b, byte c, byte d)
    {
      int result = (a<<24) | (b<<16) | (c<<8) | d;
      return result;
    }
  }

  public void decode_hex_data(String data, int[] test_data)
  {
    //decode the values
    String COMMA_DELIMITER = ",";
    String[] values = data.split(COMMA_DELIMITER);
    int[] user_data =  new int[values.length];
    for (int i=0;i<values.length;i++)
    {
      System.out.print(values[i]);
      System.out.print(" is decoded TO ");
      user_data[i] = Integer.parseInt(values[i],16);
      System.out.print(user_data[i]);
      System.out.print(" vs ");
      System.out.println(test_data[i]);
    }
  }

  public String encode_to_hex(int[] script)
  {
    String result="";
    for (int i=0;i<script.length;i++)
    {
      byte[] value = VMT.unpack32to4x8(script[i]);
      String valu = Integer.toHexString(script[i]);
      result = result + (valu);
      if (i<script.length-1) result = result + ',';
    }
    return result;
  }

  public byte decode_instrument(String inst_id)
  {
    switch (inst_id)
    {
      case "INST_ADC": return 0x01;
      case "INST_GPS": return 0x02;
      case "INST_IMG": return 0x03;
      case "INST_FPU": return 0x04; //load a constant to FPU register
      case "INST_SDR": return 0x05; //not supported yet
      case "INST_NMF": return 0x06; //set or get NMF-related parameter
      case "INST_VXM": return 0x07; //set or get internal VM parameter
    }
    return -1;
  }

  public byte decode_parameter(String param_id)
  {
    switch (param_id)
    {
      case "P_ADC_MODE": return 0x01;
      case "P_ADC_MAGX": return 0x02;
      case "P_ADC_MAGY": return 0x03;
      case "P_ADC_MAGZ": return 0x04;
      case "P_ADC_SUNX": return 0x05;
      case "P_ADC_SUNY": return 0x06;
      case "P_ADC_SUNZ": return 0x07;
      case "P_ADC_ANGX": return 0x08;
      case "P_ADC_ANGY": return 0x09;
      case "P_ADC_ANGZ": return 0x0A;
      case "P_ADC_QTNA": return 0x0B;
      case "P_ADC_QTNB": return 0x0C;
      case "P_ADC_QTNC": return 0x0D;
      case "P_ADC_QTND": return 0x0E;
      case "P_ADC_MTQX": return 0x0F;
      case "P_ADC_MTQY": return 0x10;
      case "P_ADC_MTQZ": return 0x11;
      case "P_IMG_GAIN_R": return 0x01;
      case "P_IMG_GAIN_G": return 0x02;
      case "P_IMG_GAIN_B": return 0x03;
      case "P_IMG_EXPOSE": return 0x04;
      case "P_IMG_STATUS": return 0x05; //not to be used?
      case "P_IMG_NUMBER": return 0x06;
      case "P_GPS_LATT": return 0x01;
      case "P_GPS_LONG": return 0x02;
      case "P_GPS_ALTT": return 0x03;
      case "P_GPS_TIME": return 0x04;
      case "P_NMF_TIME": return 0x01;
      case "P_VXM_TIME": return 0x01;
      case "P_VXM_PRSN": return 0x02;
      case "P_VXM_TLSC": return 0x03;
      case "P_VXM_DBUG": return 0x04;
      case "P_FPU_NIL": return 0x00;
      case "P_FPU_ONE": return 0x01;
      case "P_FPU_EXP": return 0x02;
      case "P_FPU_PIE": return 0x03;
    }
    return -1;
  }

  public byte decode_address(String addr_id)
  {
    byte var_address = (byte)Integer.parseInt(addr_id);
    return var_address;
  }

  public byte decode_action(String action_id)
  {
    switch (action_id)
    {
      case "A_IMG_DO_JPG": return 0x07;
      case "A_IMG_DO_RAW": return 0x08;
      case "A_IMG_DO_BMP": return 0x09;
      case "A_IMG_DO_PNG": return 0x0A;
      case "A_ADC_NADIR": return 0x05;
      case "A_ADC_TOSUN": return 0x06;
      case "A_ADC_BDOTT": return 0x07;
      case "A_ADC_TRACK": return 0x08;
      case "A_ADC_UNSET": return 0x09;
    }
    return -1;
  }

  public byte decode_prefix(String prefix_id)
  {
    switch (prefix_id)
    {
      /* prefixes */
      case "PRE_MOV_REG": return 0x01;
      case "PRE_MOV_RAM": return 0x02;
      case "PRE_MOV_IND": return 0x03;
      case "PRE_STR_ALU": return 0x01;
      case "PRE_STR_FPU": return 0x02;
      case "PRE_STR_BIN": return 0x03;
      case "PRE_NORMAL": return 0x01;
      case "PRE_INVERT": return 0x02;
    }
    return -1;
  }

  public byte decode_register(String reg_id)
  {
    switch (reg_id)
    {
      case "IREG_A": return 0x00;
      case "IREG_B": return 0x01;
      case "IREG_C": return 0x02;
      case "IREG_D": return 0x03;
      case "IREG_E": return 0x04;
      case "IREG_F": return 0x05;
      case "IREG_G": return 0x06;
      case "IREG_H": return 0x07;
      case "IREG_I": return 0x08;
      case "IREG_J": return 0x09;
      case "IREG_K": return 0x0A;
      case "IREG_L": return 0x0B;
      case "IREG_M": return 0x0C;
      case "IREG_N": return 0x0D;
      case "IREG_P": return 0x0E;
      case "IREG_U": return 0x0F;
      // FPU registers
      case "FREG_A": return 0x10;
      case "FREG_B": return 0x11;
      case "FREG_C": return 0x12;
      case "FREG_D": return 0x13;
      case "FREG_E": return 0x14;
      case "FREG_F": return 0x15;
      case "FREG_G": return 0x16;
      case "FREG_H": return 0x17;
      case "FREG_I": return 0x18;
      case "FREG_J": return 0x19;
      case "FREG_K": return 0x1A;
      case "FREG_L": return 0x1B;
      case "FREG_M": return 0x1C;
      case "FREG_N": return 0x1D;
      case "FREG_P": return 0x1E;
      case "FREG_U": return 0x1F;
    }
    return -1;
  }

  public byte decode_operator(String op_id)
  {
    switch (op_id)
    {
      case "ALU_EQ": return 0x01;
      case "ALU_NE": return 0x02;
      case "ALU_GT": return 0x03; //greater
      case "ALU_LT": return 0x04; //lesser
      case "ALU_GE": return 0x05;
      case "ALU_LE": return 0x06; //lesser or equal
      case "FPU_EQ": return 0x07; //same but for FPU
      case "FPU_NE": return 0x08;
      case "FPU_GT": return 0x09;
      case "FPU_LT": return 0x0A;
      case "TSX_EQ": return 0x0D; //task result is equal ...
      case "TSX_NE": return 0x0E; //task result is not equal to ...
    }
    return -1;

  }

  public int process_line(String line, int mode)
  {
    int i_mode = mode;
    int bytecode=0;
    byte op_a;
    byte op_b;
    byte op_c;
    String[] values = line.split(",");
    switch (i_mode)
    {
      case 0:
        byte group_id = (byte)Integer.parseInt(values[0].trim());
        byte task_id = (byte)Integer.parseInt(values[1].trim());
        byte freq = (byte)Integer.parseInt(values[2].trim());
        byte length = (byte)Integer.parseInt(values[3].trim());
        i_mode++;
        bytecode = VMT.pack4x8to32(group_id,task_id,freq,length);
        System.out.print(Integer.toHexString(bytecode));
        break;
      case 1:
        String opcode = values[0].trim();
        switch (opcode)
        {
          case "OP_HLT":
            bytecode = VMT.pack4x8to32(OP_HLT, OP_NOP, OP_NOP, OP_NOP);
            System.out.print(Integer.toHexString(bytecode));
            i_mode++;
            break;
          case "OP_NOP":
            bytecode = VMT.pack4x8to32(OP_NOP, OP_NOP, OP_NOP, OP_NOP);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_LEA":
            op_a = decode_register(values[1].trim());
            op_b = decode_address(values[2].trim());
            op_c = decode_address(values[3].trim());
            bytecode = VMT.pack4x8to32(OP_LEA, op_a, op_b, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_MOV":
            op_a = decode_prefix(values[1].trim());
            op_b = decode_register(values[2].trim());
            if (op_a == PRE_MOV_REG)
            {
              op_c = decode_register(values[3].trim());
              bytecode = VMT.pack4x8to32(OP_MOV, op_a, op_b, op_c);
            }
            if (op_a == PRE_MOV_RAM)
            {
              op_c = decode_address(values[3].trim());
              bytecode = VMT.pack4x8to32(OP_MOV, op_a, op_b, op_c);
            }
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_CMP":
            op_a = decode_operator(values[1].trim());
            op_c = decode_register(values[3].trim());
            if ((op_a == TSX_EQ) | (op_a == TSX_NE))
            {
              op_b = decode_address(values[2].trim());
              bytecode = VMT.pack4x8to32(OP_CMP, op_a, op_b, op_c);
            }
            else
            {
              op_b = decode_register(values[2].trim());
              bytecode = VMT.pack4x8to32(OP_CMP, op_a, op_b, op_c);
            }
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_GET":
            op_a = decode_instrument(values[1].trim());
            op_b = decode_parameter(values[2].trim());
            op_c = decode_register(values[3].trim());
            bytecode = VMT.pack4x8to32(OP_GET, op_a, op_b, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_SET":
            op_a = decode_instrument(values[1].trim());
            op_b = decode_parameter(values[2].trim());
            op_c = decode_register(values[3].trim());
            bytecode = VMT.pack4x8to32(OP_SET, op_a, op_b, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_ACT":
            op_a = decode_instrument(values[1].trim());
            op_b = decode_action(values[2].trim());
            op_c = decode_register(values[3].trim());
            bytecode = VMT.pack4x8to32(OP_ACT, op_a, op_b, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_STR":
            op_a = decode_prefix(values[1].trim());
            op_c = decode_register(values[2].trim());
            bytecode = VMT.pack4x8to32(OP_STR, op_a, OP_NOP, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_FMA":
            op_a = decode_register(values[1].trim());
            op_b = decode_register(values[2].trim());
            op_c = decode_register(values[3].trim());
            bytecode = VMT.pack4x8to32(OP_FMA, op_a, op_b, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_FSD":
            op_a = decode_register(values[1].trim());
            op_b = decode_register(values[2].trim());
            op_c = decode_register(values[3].trim());
            bytecode = VMT.pack4x8to32(OP_FSD, op_a, op_b, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_SIN":
            op_a = decode_prefix(values[1].trim());
            op_b = decode_register(values[2].trim());
            op_c = decode_register(values[3].trim());
            bytecode = VMT.pack4x8to32(OP_SIN, op_a, op_b, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_COS":
            op_a = decode_prefix(values[1].trim());
            op_b = decode_register(values[2].trim());
            op_c = decode_register(values[3].trim());
            bytecode = VMT.pack4x8to32(OP_COS, op_a, op_b, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_TAN":
            op_a = decode_prefix(values[1].trim());
            op_b = decode_register(values[2].trim());
            op_c = decode_register(values[3].trim());
            bytecode = VMT.pack4x8to32(OP_TAN, op_a, op_b, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_POW":
            op_a = decode_prefix(values[1].trim());
            op_b = decode_register(values[2].trim());
            op_c = decode_register(values[3].trim());
            bytecode = VMT.pack4x8to32(OP_POW, op_a, op_b, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case "OP_NOR":
            op_a = decode_register(values[1].trim());
            op_b = decode_register(values[2].trim());
            op_c = decode_register(values[3].trim());
            bytecode = VMT.pack4x8to32(OP_NOR, op_a, op_b, op_c);
            System.out.print(Integer.toHexString(bytecode));
            break;
        }
        break;
      case 2:
        String s_ivalue = line.substring(0,line.length() - 1);
        switch(line.charAt(line.length() - 1))
        {
          case 'i':
            bytecode = Integer.parseInt(s_ivalue);
            System.out.print(Integer.toHexString(bytecode));
            break;
          case 'f':
            float fvalue = Float.parseFloat(s_ivalue);
            bytecode = Float.floatToIntBits(fvalue);
            System.out.print(Integer.toHexString(bytecode));
            break;
        }
        break;
    }
    return i_mode;
  }

  public void read_source_file(String filename)
  {
    try {
    FileInputStream fstream = new FileInputStream(filename);
      BufferedReader breader = new BufferedReader(new InputStreamReader(fstream));
      String line = "";
      int mode = 0;
      Boolean first_line = true;
      do {
        line = breader.readLine();
        if (line != null)
        {
          if (line.indexOf("//")<0)
          {
            if (!first_line) System.out.print(",");
            mode = process_line(line, mode);
            first_line = false;
          }
        }
      } while (line!=null);

      fstream.close();
      System.out.println();
    }
    catch (FileNotFoundException ex)
    {
      System.out.println("File not found!");
    }
    catch (IOException ex)
    {
      System.out.println("I/O error!");
    }
  }

  public static void main(final String args[])
  {
    SpliceTranslator VMSAssembler = new SpliceTranslator();
    String filename="";
    if (args.length>1)
    {
      if (args[0].equals("-s"))
      {
        filename = args[1];
        VMSAssembler.read_source_file(filename);
      }
    }
  }
}
