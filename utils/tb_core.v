`timescale 1ps/1ps

module tb_core;
	// Steps scale
	parameter		STEP = 10;

	// Memory dump enable
	parameter MEMDUMP = "TRUE";

	// clock and reset signal(initialize zero)
	reg				clk=0, rst=0;


	// instruction memory interface
	// memd: memory data, mema: memory address
	wire	[31:0]	w_imemd, w_imema;


	// data memory interface
	// wdata: write data, rdata: read data
	// wen: write enable, ren: read enable(unused)
	// dmema: data memory address
	wire	[31:0]	w_wdata, w_rdata;
	wire			w_wen, w_ren;
	wire	[31:0]	w_dmema; 

	// data memory
	reg [31:0] mem[0:1024*8-1];


	// clock signal generator
	always #(STEP/2) begin
		clk	<=	~clk;
	end

	// ----- instance -----
	// instruction memory
	imem im(
		.i_addr	(w_imema),
		.o_inst	(w_imemd)
	);
	// data memory
	assign	w_rdata = mem[w_dmema[14:2]];

	always @(posedge clk) begin
		if (w_wen) begin
			mem[w_dmema[14:2]]	<=	w_wdata;
		end
	end

	core #(
	.RVM("TRUE"),
	.RVV("TRUE"),
	.VLEN(128)
	)cpu(
		.clk(clk),
		.rst(rst),

		.i_exstall(1'b0),

		.i_interrupt(1'b0),

		.i_inst(w_imemd),
		.o_iaddr(w_imema),

		.i_read_data(w_rdata),
		.o_read_en(w_ren),
		.i_read_vd(1),
		.o_write_data(w_wdata),
		.o_write_en(w_wen),
		.o_memaddr(w_dmema)
	);

	// Initialize and definition of finish condition
	// Edit
	initial begin
		rst		=	1;
		#(STEP*5);
		rst		=	0;
		#(STEP*500);
		$finish;
	end

	// file generate for gtkwave
	integer idx; // need integer for loop
	initial begin
		//$monitor ($stime, " inst = %8x", w_imemd);
		$dumpfile("tmp/dump.vcd");
		$dumpvars(0, tb_core);
		if (MEMDUMP == "TRUE") begin
			for (idx = 0; idx < 1024; idx = idx + 1) $dumpvars(1, mem[idx]);
		end	
	end
	//

endmodule