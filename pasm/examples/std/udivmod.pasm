%ifndef std_udivmod_pasm
%define std_udivmod_pasm

;------------ udivmod ------------;
;              input              ;
;  rx            - dividend       ;
;  ry            - divisor        ;
;                                 ;
;              output             ;
;  rx            - remainder      ;
;  ry            - <override>     ;
;  rz            - quotient       ;
;---------------------------------;

udivmod:
                      ; let the remainder be the dividend
 put rz,0             ; let the quotient be zero

udivmod_loop:
 psh rx               ; while the remainder is greater than or equal to the divisor,
 psh ry
 psh ret_JG8M
 jmp uge ret_JG8M:
 pop ry
 jpz udivmod_end,rx
 pop rx

 sub rx,ry            ; subract the divisor from the remainder,

 inc rz               ; increment the quotient

 jmp udivmod_loop     ; and repeat

udivmod_end:
 pop rx
 
 ret

%include "uge.pasm"

%end