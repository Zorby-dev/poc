%ifndef std_parseu_pasm
%define std_parseu_pasm

;------------- parseu -------------;
;    parses a u8 from a string     ;
;                                  ;
;               input              ;
;  rx             - &string        ;
;                                  ;
;              output              ;
;  rx             - <override>     ;
;  ry             - result         ;
;  rz             - <override>     ;
;----------------------------------;

parseu:
 put rz,1              ; initialize the multiplier
 str parseu_mul,rz

 put rz,0              ; initialize the accumulator
 str parseu_acc,rz

parseu_loop:
 ldr ry,rx             ; load character from buffer

 jpz parseu_end,ry     ; if at end of string, jump to end
 psh rx

 put rz,'0'            ; else, transform character into digit
 sub ry,rz

 put rx,ry             ; multiply the digit by the multiplier
 ldr ry,parseu_mul
 psh ret_URJY
 jmp mul ret_URJY:

 ldr ry,parseu_acc     ; add the number to the accumulator
 add ry,rz
 str parseu_acc,ry

 ldr ry,parseu_mul     ; multiply the multiplier by 10
 put rx,10
 psh ret_85XC
 jmp mul ret_85XC:
 str parseu_mul,rz

 pop rx                ; and repeat
 inc rx
 jmp parseu_loop

parseu_end:
 ldr ry,parseu_acc     ; return
 ret

parseu_mul: 0
parseu_acc: 0

%include "mul.pasm"

%end