;;; definitions ;;;


%include "devices.pasm"
%include "std/ptes.pasm"

%define KEY_UP    17
%define KEY_LEFT  30
%define KEY_DOWN  31
%define KEY_RIGHT 32

%define POS_MASK  0b0111_0111

%define DIR_UP    0b0111_0000
%define DIR_LEFT  0b0000_0111
%define DIR_DOWN  0b0001_0000
%define DIR_RIGHT 0b0000_0001


;;; program ;;;


; this is the main loop of the program
; it is divided into two steps:
;  - events  (responds to keyboard input)
;  - update  (updates the game state)
;  - draw    (draws the game state on the screen)

loop:                 ; (events)

 ldr rz,head_dir      ; store the head direction for future use

 ldr rx,KEYBOARD      ; get the currently pressed key (0 indicates no pressed keys)

 jpz update,rx        ; if no keys are down, continue

 ldr ry,prev_key
 jpz pressed,ry       ; if no keys were down in the previous iteration,
                      ; that means a key was pressed in this iteration

 jmp update           ; otherwise, a key was already held down
                      ; => skip

pressed:              ; responds to pressed keys
 put ry,KEY_UP
 sub ry,rx
 jpz pressed_up,ry

 put ry,KEY_LEFT
 sub ry,rx
 jpz pressed_left,ry

 put ry,KEY_DOWN
 sub ry,rx
 jpz pressed_down,ry

 put ry,KEY_RIGHT
 sub ry,rx
 jpz pressed_right,ry

update:
 str prev_key,rx      ; save data before drawing
 str head_dir,rz
 
 ldr ry,ticks         ; advance the tick count
 inc ry
 str ticks,ry

 jpz move,ry          ; every 256 ticks, we move the snake
                      ; (so that it's not extremely fast and easier to control)

 jmp draw

move:
 ldr ry,big_ticks     ; increment the upper byte (explanation near definiton)
 inc ry
 str big_ticks,ry

 ldr rx,head_pos      ; move the snake in the specified direction
 add rx,rz

 and rx,POS_MASK      ; this step wraps the position around if it's out of bounds
                      ; (see explanation above the definition of the `head_pos` label)

 str head_pos,rx      ; save the head pos

 ldr rz,apple_pos     ; check for head-apple collision
 sub rx,rz
 jpz eat_apple,rx

draw:
 put rx,0             ; this is the pixel iterator

draw_loop:
 put rz,rx            ; transform the pixel iterator into it's current position
 and rz,0b00_111_000  ; (see the comments above the `head_pos` label)
 add rz,rx            ; the pixel iterator is in the format `00_yyy_xxx`,
                      ; so by adding the y component of itself to it,
                      ; we effectively shift it by one bit
                      ; (so it looks like this: `0_yyy_0_xxx`), which is what a pos looks like

 ldr ry,head_pos      ; now that we stored the current pixel position into rz,
 sub ry,rz            ; we can easily compare it with other position values
 jpz draw_head,ry

 ldr ry,apple_pos
 sub ry,rz
 jpz draw_apple,ry

 put rz,C_SPACE       ; if neither the head nor the apple are at the current position,
 jmp draw_square      ; we draw an empty space

draw_apple:
 put rz,150           ; print special PTES characters to draw the apple
 str STI,rz
 put rz,151
 str STI,rz

 jmp pixel_end

draw_head:
 put rz,168           ; print the special character INVERTED_CAPITAL_O,
                      ; because it looks like an eye :^)

draw_square:
 str STI,rz           ; STI characters have an aspect ratio of 1:2, so by printing
 str STI,rz           ; them twice, we get a square

pixel_end:
 inc rx               ; increment the pixel iterator (move to next pixel)

 put rz,64            ; the amount of pixels inside STI divided by 2
                      ; (because of the same 1:2 ratio)

 sub rz,rx            ; if we drew the entire screen, we jump back to the top
 jpz loop,rz

 jmp draw_loop        ; otherwise we draw the next pixel

;; various subroutines ;;

pressed_up:
 put rz,DIR_UP
 jmp update

pressed_left:
 put rz,DIR_LEFT
 jmp update

pressed_down:
 put rz,DIR_DOWN
 jmp update

pressed_right:
 put rz,DIR_RIGHT
 jmp update

eat_apple:
 and ry,23            ; arbitrary operations to sort of imitate random number generation
rng_loop:
 add rz,67
 dec ry
 jpn rng_loop_end,ry
 jmp rng_loop
rng_loop_end:

 and rz,POS_MASK      ; and clamp it again

 str apple_pos,rz     ; store the new "random" position

 jmp draw             ; and continue


;;; data ;;;


prev_key:  0          ; the key that was pressed in the previous iteration

ticks:     0          ; the amount of passed ticks

big_ticks: 0          ; used for "rng". you can think of these two variables as
                      ; one 16 bit number, with `ticks` being the lower 8 bits
                      ; and `big_ticks` being the upper 8 bits

; position structure:
;  0 yyy 0 xxx
;  │ │   │ ╰──── x
;  │ │   │
;  │ ╰───│────── y
;  │     │
;  ╰─────┴────── buffers

; i chose this layout because:
;  - is more memory efficient than using two bytes

;  - is cheaper to compare

;  - allows for fast modifying of both values (see below)

;  - thanks to the buffer zeros allows for quick wrapping around by anding position with 0b0111_0111

;  - or by anding it with the opposite allows for
;    extremely fast wall-collision checking, for example:

;     ; (rx stores position)
;     and rx,0b1000_1000
;     jpz no_collision,rx
;     ; if we didn't jump, that means there was a collision

;    (these two only require 1-2 instructions and therefore are WAAY cheaper than divmod / comparisons)

; modifying position:
; - adding   1 increases x
; - adding   7 decreases x (overflows)
; - adding  16 increases y
; - adding 112 decreases y (overflows)

head_pos:  0b0000_0000 ; POS 0,0
head_dir:  DIR_RIGHT

apple_pos: 0b0100_0100 ; POS 4,4