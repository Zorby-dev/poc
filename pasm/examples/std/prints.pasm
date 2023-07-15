%ifndef std_prints_pasm
%define std_prints_pasm

;-------------- prints -------------;
;  prints a string to the terminal  ;
;                                   ;
;               input               ;
;  rx             - &string         ;
;                                   ;
;               output              ;
;  rx             - <override>      ;
;  ry             - <override>      ;
;-----------------------------------;

prints:
loop:
 ldr ry,rx      ; load character to ry

 jpz end,ry     ; if at end of string, jump to end

 str OUT,ry     ; else, send character to terminal

 inc rx         ; and repeat
 jmp loop

end:
 ret

%end