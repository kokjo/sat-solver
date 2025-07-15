#!/bin/sh
yosys -p "read_verilog adder.v; synth; abc -g AND,NAND,OR,NOR,XOR,XNOR; write_json adder.json"