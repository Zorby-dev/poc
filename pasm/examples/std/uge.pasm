%ifndef std_uge_pasm
%define std_uge_pasm

;------------- uge -------------;
;             input             ;
;  rx           - greater       ;
;  ry           - lesser        ;
;                               ;
;             output            ;
;  rx           - result        ;
;-------------------------------;

uge:
 jpn uge_rx_ge_128,rx
 jpn uge_false,ry
 jmp uge_sub

uge_rx_ge_128:
 jpn uge_sub,ry
 jmp uge_true

uge_sub:
 sub rx,ry
 jpn uge_false,rx
 jmp uge_true

uge_false:
 put rx,0
 jmp uge_end

uge_true:
 put rx,1

uge_end:
 ret

%end