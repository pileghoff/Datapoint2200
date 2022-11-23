# Datapoint 2200 emulator

This is my attempt to bring the Datapoint 2200 to life!
I have never touched machine, nor have i ever seen another emulator, but the machine fascinates me, and i thought my have a go at it.

The goal is to implement version 2 of the processor, with enough precision to run the original Cassette Tape Operating System.

I am working with this [Reference Manual](http://bitsavers.org/pdf/datapoint/2200/Datapoint_2200_Reference_Manual.pdf) from bitsavers as the ultimate source of truth, but others sources of knowledge i rely on are:

- [Programmers Manuel](http://bitsavers.org/pdf/datapoint/2200/2200_Programmers_Man_Aug71.pdf), this is from before version 2, but contains good hints
- [A history lesson by Gordon Peterson](http://bitsavers.org/pdf/datapoint/history/Gordon_Peterson_DP_history.txt), other than being a fascianting look into the history of the machine, this contains some pieces of information that are hard (impossible?) to figure out by just reading the official manuals

## Assembler

I have also implemented an assembler. This does not attempt to recreate the original assembler in any way. It is a way to generate test programs, so i don't have to create the binaries by hand. It is bare bones, but it works.
