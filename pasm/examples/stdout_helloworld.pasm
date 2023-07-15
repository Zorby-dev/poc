%include "devices.pasm"       ; include device definitions

 psh ret                      ; push return address on the stack
 put rx,data                  ; store string pointer to rx
 jmp prints ret:              ; call `prints`
 hlt                          ; halt the program

data:
 'H''e''l''l''o'','' ''W''o''r''l''d''!''\n'0

%include "std/prints.pasm"    ; include function for printing strings