module Basics exposing (..)

infix non   4 (<)  = lt
infix non   4 (>)  = gt
infix right 5 (++) = append
infix left  6 (+)  = add
infix left  6 (-)  = sub
infix left  7 (*)  = mul

lt =
    Elm.Kernel.Basics.lt

gt =
    Elm.Kernel.Basics.gt

append =
    Elm.Kernel.Basics.append

add =
    Elm.Kernel.Basics.add

sub =
    Elm.Kernel.Basics.sub

mul =
    Elm.Kernel.Basics.mul
