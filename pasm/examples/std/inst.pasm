%ifndef std_inst_pasm
%define std_inst_pasm

%ifndef __FEATURE_STACK
 %ifndef STACK_CAP
  %define STACK_CAP 5
 %end

 inst_stack_begin: %repeat STACK_CAP 0x00

 inst_stack_ptr: inst_stack_begin
 inst_stack_temp: 0x00

 %macro psh dst
  %ifeq dst rx
   %define REG ry
  %else
   %define REG rx
  %end

  str inst_stack_temp,REG
  ldr REG,inst_stack_ptr

  str REG,dst
  inc REG

  str inst_stack_ptr,REG
  ldr REG,inst_stack_temp
 %end

 %macro pop dst
  %ifeq dst rx
   %define REG ry
  %else
   %define REG rx
  %end

  str inst_stack_temp,REG
  ldr REG,inst_stack_ptr

  dec REG
  ldr dst,REG

  str inst_stack_ptr,REG
  ldr REG,inst_stack_temp
 %end
%end

%end