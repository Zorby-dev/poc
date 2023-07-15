%include "devices.pasm"

 put rx,0x00

loop:
 str STI,rx
 inc rx

 put ry,0xff
 sub ry,rx
 jpz end,ry

 jmp loop

end:
 hlt