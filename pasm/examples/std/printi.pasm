%ifndef std_printi_pasm
%define std_printi_pasm

;------------- printi ------------;
;  prints an i8 to the terminal   ;
;                                 ;
;              input              ;
;  rx            - byte to print  ;
;                                 ;
;              output             ;
;  rx            - <override>     ;
;  ry            - <override>     ;
;  rz            - <override>     ;
;---------------------------------;

printi: ; TODO: fix edge case with -128
 put ry,10            ; divmod the number by 10 to get the digit
 jpn printi_negative,rx
printi_recursion:
 psh ret_OLDR
 jmp udivmod ret_OLDR:

 psh rx               ; store digit to be printed

 jpz printi_end,rz    ; if this is the last digit, start printing

 put rx,rz            ; else continue recursively
 psh printi_end
 jmp printi_recursion

printi_negative:
 neg rx
 put rz,'-'
 str OUT,rz

 jmp printi_recursion

printi_end:
 pop rx               ; print the digit
 put ry,'0'
 add rx,ry
 str OUT,rx

 ret

%include "udivmod.pasm"

%end