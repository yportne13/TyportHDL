# TyportHDL

the target of this project:

* an interpreter for a basic part of SpinalHDL

* a language server that special for SpinalHDL (add width and clockdomain when hover on a signal. all the check such as `Latch Detect` `Width mismatch` can be done when writing instead of when generating verilog code)

* package manager and build system. It is still a little difficult for SpinalHDL user work with Verilog/sv/VHDL. and there is no usefull package manage and build system for Verilog/sv/VHDL user. all these problems can be solved by creating a package manager and build system for both SpinalHDL and Verilog/sv/VHDL.

* add more grammars which is difficult to add on scala. In rust, there is a thing called lifetime annotation. it's realy useful for compiler to check more errors in your code. In HDL, maybe we can add some annotations on the io signal, such as, which clk this signal belong to, and which pair of signals should active on the same time(such as, Flow, the valid and payload should always align).
