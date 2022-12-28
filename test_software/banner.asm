# Select CRT on the databus
LoadImm A, 0xe1
Adr
Data

# Set the cursor position to line 6
LoadImm A, 6
Com3

# Set the cursor to char 20
LoadImm A, 20
LoadImm B, 20
Com2

# Print Hello World!
# Start by setting the HL register to point to the beginning of the string
# For each loop:
# - Write the data from memory to the CRT
# - Add 1 to the memory pointer
# - Move the cursor 1
# - If the memory pointed to is not zero, loop back
LoadImm H, 0
LoadImm L, string

loop: 
# Write data to crt
Load A, M
Write
# Move data pointer 1
Load A, L
AddImm 1
Load L, A
# Move cursor 1
Load A, B
AddImm 1
Load B, A
Com2
# Check data
Load A, M
SubImm 1
JumpIfNot Cf, loop
Halt

string: DATA "Hello World!", 0