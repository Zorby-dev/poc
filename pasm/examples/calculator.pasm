%include "devices.pasm"

loop:
 psh ret_30S7                 ; read first number
 jmp read_number ret_30S7:
 psh ry

 psh rx                       ; store the operator

 psh ret_IF7L                 ; read second number
 jmp read_number ret_IF7L:

 ldr rx,IN

 pop rx

 put rz,'+'                   ; check if operator is +
 sub rz,rx
 jpz add,rz

 put rz,'-'
 sub rz,rx
 jpz sub,rz                   ; check if operator is -

 put rz,'*'
 sub rz,rx
 jpz mul_,rz

 put rz,'/'
 sub rz,rx
 jpz div,rz

 put rz,'%'
 sub rz,rx
 jpz mod,rz

add:
 pop rx
 add rx,ry
 jmp print

sub:
 pop rx
 sub rx,ry
 jmp print

mul_:
 pop rx
 psh ret_85YC
 jmp mul ret_85YC:
 put rx,rz
 jmp print

div:
 pop rx
 psh ret_ZWO2
 jmp udivmod ret_ZWO2:
 put rx,rz
 jmp print

mod:
 pop rx
 psh print
 jmp udivmod

print:
 psh ret_DA2G
 jmp printu ret_DA2G:
 put rx,'\n'
 str OUT,rx
 jmp loop

; output:
; rx - first non-numeric character
; ry - parsed number
; rz - <override>
read_number:
 put rx,parse_buffer_end
read_number_loop:
 put rz,'0'
 ldr ry,IN
 psh ry
 sub ry,rz
 jpn read_number_end, ry
 put rz,0x76
 add ry,rz
 jpn read_number_end, ry
 pop ry
 dec rx
 str rx,ry
 jmp read_number_loop
read_number_end:
 psh ret_63VA
 jmp parseu ret_63VA:
 pop rx
 ret

0 0 0 parse_buffer_end: 0

%include "std/mul.pasm"
%include "std/udivmod.pasm"
%include "std/parseu.pasm"
%include "std/printu.pasm"