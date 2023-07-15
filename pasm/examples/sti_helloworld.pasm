%include "std/ptes.pasm"      ; include PTES charset definitions
%include "devices.pasm"       ; include device definitions

 put rx,string                ; store string pointer to rx
 psh ret                      ; push return address on the stack
 jmp prints ret:              ; call `prints`
 hlt                          ; halt the program

string:
 C_H C_e C_l C_l C_o C_COMMA C_SPACE C_W C_o C_r C_l C_d C_BANG C_NULL

%define OUT STI               ; define the output device for `prints` to use

%include "std/prints.pasm"    ; include function for printing strings