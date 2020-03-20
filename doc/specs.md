Current (OPS-SAT version of Splice ) minimal language specifications

variables:
- only simple, non-compound typed variables are possible i.e
- var variable1:i32 or var variable2:f32 are acceptable declarations
constants:
- same as variables
types:
- two types supported: "i32" (signed integer,) and "f32"
expressions:
- basic arithmetic and trigonometric operations and assignment operator
- Refer to Splice whitepaper for full list of operations supported by VM assembly
keywords (task list file only):
- group, task, data, freq, preq, exec, var, f32, i32, const, return
keywords (instrument definition file only):
- inst, queue, size, type, accs, shed, var, prop

OPS-SAT instrument specifications
- Camera, GPS and ADCS instruments are available at the moment - instrument definition file TBD


Compilation process imagined:
- Instrument definition files (.spli extension) are provided from the satellite operations team
- Group/task list in a source file (.spl extension) are provided from software developers
- Each task is compiled into a single assembly file (.splc)
- Linker can produce a semi-binary executable file compatible with VM OPS-SAT from one or many assembly files (.splx)
- Source: (.spl+.spli) -> assembly representation (.splc) -> executable format (.splx)
- For future satellite missions  both assembly and executable formats can and should change, but high-level syntax should't
- Each satellite may have its own instrument definition file (.spli), slightly or significantly different from each other
