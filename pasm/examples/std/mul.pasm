%ifndef std_mul_pasm
%define std_mul_pasm

;------------- mul -------------;
;             input             ;
;  rx           - multiplicand  ;
;  ry           - multiplier    ;
;                               ;
;             output            ;
;  ry           - `0`           ;
;  rz           - result        ;
;-------------------------------;

mul:
 put rz,0          ; multiplication accumulator

mul_loop:
 jpz mul_end,ry    ; if this was the last iteration, jump to end

 add rz,rx         ; else, add the multiplicand to the accumulator

 dec ry            ; and repeat
 jmp mul_loop

mul_end:
 ret

%end