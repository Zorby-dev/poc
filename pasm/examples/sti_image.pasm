%include "devices.pasm"
%include "std/ptes.pasm"

 put rx,board_begin    ; print the board:

draw_loop:
 put ry,board_end      ; if at end of board,
 sub ry,rx
 jpz end,ry            ; jump to end

 ldr ry,rx             ; else, convert the current cell to a character,
 jpz empty_cell,ry

 put ry,C_FILLED
 jmp draw_cell

empty_cell:
 put ry,C_SPACE

draw_cell:
 str STI,ry            ; draw it
 str STI,ry            ; (twice, because STI characters are 6x12 pixels, and we want a 12x12 cube)

 inc rx                ; and repeat
 jmp draw_loop

end:
 hlt

board_begin:
 0 0 0 0 0 0 0 0
 0 0 1 0 0 1 0 0
 0 0 1 0 0 1 0 0
 0 0 1 0 0 1 0 0
 0 0 0 0 0 0 0 0
 0 1 0 0 0 0 1 0
 0 0 1 1 1 1 0 0
 0 0 0 0 0 0 0 0
board_end: