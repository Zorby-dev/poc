%ifndef std_ige_pasm
%define std_ige_pasm

;------------- ige -------------;
;             input             ;
;  rx           - greater       ;
;  ry           - lesser        ;
;                               ;
;             output            ;
;  rx           - result        ;
;-------------------------------;

ige:
 jpn ige_rx_neg,rx
 jpn ige_true,ry
 jmp ige_sub

ige_rx_neg:
 jpn ige_sub,ry
 jmp ige_false

ige_sub:
 sub rx,ry
 jpn ige_false,rx
 jmp ige_true

ige_false:
 put rx,0
 jmp ige_end

ige_true:
 put rx,1

ige_end:
 ret

%end