%ifndef std_printu_pasm
%define std_printu_pasm

;------------- printu ------------;
;   prints a u8 to the terminal   ;
;                                 ;
;              input              ;
;  rx            - byte to print  ;
;                                 ;
;              output             ;
;  rx            - <override>     ;
;  ry            - <override>     ;
;  rz            - <override>     ;
;---------------------------------;

printu:
 put ry,10            ; divmod the number by 10 to get the digit
 psh ret_OLDR
 jmp udivmod ret_OLDR:

 psh rx               ; store digit to be printed

 jpz printu_end,rz    ; if this is the last digit, start printing

 put rx,rz            ; else continue recursively
 psh printu_end
 jmp printu

printu_end:
 pop rx               ; print the digit
 put ry,'0'
 add rx,ry
 str OUT,rx

 ret

%include "udivmod.pasm"

%end