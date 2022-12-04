# Understanding the machine

This document holds my notes, as i try to understand how the machine functions in details.
The things that in the reference manual are unclear i will document here as i try to unravel the mysteries.
Not all my conclusions will be correct, so i will try to document my reasoning, so it's easier to spot my mistakes.

## The keyboard

The reference manual is light on information about how the keyboard is suppose to function.
If we look at the [Programmers manual](bitsavers.org/pdf/datapoint/2200/2200_Programmers_Man_Aug71.pdf) p. 8-4 (108 in the pdf) we see that it mentions that the keyboard has a 1 character buffer.

I suspect that it works as follows:
1. Pressing a key, fills the buffer with the corresponding character and sets the read ready bit.
2. Releasing a key clears the read ready bit, but does not affect the buffer.
3. Issueing the `Input` command, while the keyboard data is on the dataline, clears the read ready bit ([Programmers manual p. 8-3 s. 3.3](bitsavers.org/pdf/datapoint/2200/2200_Programmers_Man_Aug71.pdf))

But a closer look at the documentation and possibly the schematics will have to be done to confirm.