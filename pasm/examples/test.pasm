%include "devices.pasm"

 put rx,128
 psh ret
 jmp printi ret:
 put rx,'\n'
 str OUT,rx
 hlt

%include "std/printi.pasm"