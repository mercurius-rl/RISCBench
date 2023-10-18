use rv_sim::asm_f;

use core::panic;
use std::process::*;
use std::path::Path;
use std::env;

mod tes_win;

fn main() {
	let s: Vec<String> = env::args().collect();

	if s.len() < 2 {
		tes_win::fun();
		return;
	}

	let core = s.len() > 2;

	// アセンブラソースコードの所在確認
	let path = Path::new(&(s[1]));
	if !Path::is_file(path) {
		panic!("This file({}) not exist.", s[1])
	}

	// アセンブリ：バイナリへの変換
	asm_f::asm(&(s[1]), "tmp/data.dat");

	// 命令の行数に合わせて

	// iverilogでのRVシミュレーション用引数の作成
	let mut ivarg = vec!["-o", "tmp/a.out", "utils/tb_core.v", "utils/imem.v"];
	let tmp_s:String;
	if core {
		tmp_s = s[2].clone() + "/*";
		ivarg.push(&tmp_s);
	} else {
		ivarg.push("core/*");
	}
	
	let my_array: Result<[&str; 5], _> = ivarg.try_into();
	let mut conv_arg : [&str; 5]= ["","","","",""];
	match my_array {
		Ok(arr) => conv_arg = arr,
		Err(_) => println!("Failed to convert vector to array."),
	}

	// iverilogシミュレーションの実行
	let out = Command::new("iverilog")
		.args(conv_arg)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.output()
		.unwrap();

	if !out.status.success() {
		panic!("iverilog could't finish correctly.")
	} 

	let out = Command::new("vvp")
		.args(["tmp/a.out"])
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.output()
		.unwrap();

	if !out.status.success() {
		panic!("vvp could't finish correctly.")
	}
}
