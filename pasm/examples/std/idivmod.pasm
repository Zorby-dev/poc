%ifndef std_idivmod_pasm
%define std_idivmod_pasm

;------------ idivmod ------------;
;              input              ;
;  rx            - dividend       ;
;  ry            - divisor        ;
;                                 ;
;              output             ;
;  rx            - remainder      ;
;  ry            - <override>     ;
;  rz            - quotient       ;
;---------------------------------;

idivmod:
 jpn idivmod_divisor_neg,ry
 jpn idivmod_dividend_neg,rx
 psh ret_JG8N
 jmp udivmod ret_JG8N:
 jmp idivmod_end

idivmod_divisor_neg:
 neg ry
 psh ret_ZWP3
 jmp udivmod ret_ZWP3:
 jmp idivmod_neg_quotient

idivmod_dividend_neg:
 neg rx
 psh ry
 psh ret_2ZR6
 jmp udivmod ret_2ZR6:
 pop ry
 jpz idivmod_neg_quotient,rx
 neg rz ; TODO: possible optimization sub 255,rz
 dec rz
 sub ry,rx
 put rx,ry
 jmp idivmod_end

idivmod_neg_quotient:
 neg rz

idivmod_end:
 ret

%include "udivmod.pasm"

%end