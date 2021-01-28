extern crate futures;
use scanner_rust::ScannerAscii;

use scanner_rust::generic_array::typenum::U64;
use scanner_rust::Scanner as scanner2;
// use scanner_rust::ScannerStr as scanner_str;
use scanner_rust::ScannerError;

use std::env;

use futures::executor::{block_on, ThreadPool};
use futures::task::SpawnExt;
use std::process::Command;
use std::str;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);
    if (args.len() > 1) {
        let mut command_parse: Option<Vec<&str>> = None;
        // find command method
        if let Some(command) = args.get(1) {
            // find `=`
            if command.contains("command") {
                let v: Vec<&str> = command.split('=').skip(1).collect();
                if v.len() == 1 {
                    let v: Vec<&str> = v[0].split(',').collect();
                    command_parse = Some(v);
                }
            }
        }

        let mut extras_command_parse : Option<Vec<&str>> = None;
        // find command method
        if let Some(command) = args.get(2) {
            // find `=`
            if command.contains("extras") {
                let v: Vec<&str> = command.split('=').skip(1).collect();
                if v.len() == 1 {
                    let v: Vec<&str> = v[0].split(',').collect();
                    extras_command_parse = Some(v);
                }
            }
        }

        let command_parse = command_parse.unwrap_or(vec!["none"]);
        let extras_command_parse = extras_command_parse.unwrap_or(vec!["none"]);

        assert_eq!(command_parse.len(), extras_command_parse.len());

        let command_parse : Vec<(&&str, &&str)> = command_parse.iter().zip(extras_command_parse.iter()).collect();

        println!("{:?}", command_parse);

        for option in command_parse.iter() {
            let (a, extras) = option;
            match &a[..] {
                "find_bazel_log" => {
                    let pool = ThreadPool::new().expect("Failed to build pool");

                    // future that return 
                    let result = pool.spawn_with_handle(async move {
                        let mut copy_file = Command::new("pwd");

                        // construct path with real path , got from `pwd` command
                        let mut real_path = str::from_utf8(&copy_file.output().unwrap().stdout).unwrap().to_string();
                        let last_index = real_path.len()-1;
                        real_path.remove(last_index);
                        real_path.push_str("/path.log");


                        let mut scan: scanner2<_, U64> = scanner2::scan_path2(real_path).unwrap();

                        let command_log_path = scan.next_line().unwrap().unwrap();

                        let mut copy_file = Command::new("cp");

                        copy_file.arg(command_log_path).arg(".");

                        copy_file.output()
                    }).unwrap();
                    println!("result = {:?}", block_on(result));
                }
                "parse_profile_to_grafana" => {
                    // use std::io::{self, Write};
                    let total_time_regex = r"(Total\srun\stime)(\s+)([+-]?([0-9]*[.])?[0-9]+)(\s+)(ms|s)";

                     // get semua string per line 
                     // run parallel 
                     // create each futures and merge into one vec 
                     // contohnya pakai join
                     let dump_to_analyze_profile = 
                            Command::new("bazel")
                            // .arg(format!(" analyze-profile {}", &extras))
                            .arg("analyze-profile")
                            .arg(&extras)
                            // .arg(" > ")
                            // .arg("analyze-profile.txt")
                            .output()
                            .expect("failed to execute process");


                    println!("dump_to_analyze_profile: {:?}", dump_to_analyze_profile);
                    println!("status: {}", dump_to_analyze_profile.status);

                    let buf = &dump_to_analyze_profile.stdout;
                    let s = match  str::from_utf8(buf){
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                    
                    println!("result: {}", s);

                    // let mut scan: scanner_str = scanner_str::new(s);

                    // io::stdout().write_all(&dump_to_analyze_profile.stdout).unwrap();
                    // io::stderr().write_all(&dump_to_analyze_profile.stderr).unwrap();

                    // println!("parse_profile_to_grafana : {:?}", dump_to_analyze_profile.stdout);


                    use regex::Regex as raw_regex;
                    let re = raw_regex::new(r"(Total\srun\stime)(\s+)([+-]?([0-9]*[.])?[0-9]+)").unwrap();
                    
                    assert!(re.is_match("Total run time                435.463"));
                    assert!(re.is_match(s));

                    let mut vec: Vec<String> = Vec::new();
                    let mut compile_time : Option<String> = None;
                    for mat in re.captures_iter(s) {
                        println!("{}", mat.len());
                        if mat.len() >= 4 {
                            compile_time = Some(mat[3].to_owned());
                            break;
                        }
                    }

                    println!("start_time=1,fetch_duration=200,build_duration=201,report_duration=202,total_duration={:?}",compile_time);

                    let send_data = Command::new("./influxdb_add.sh")
                            .arg("bazel")
                            .arg("host=\"sekarang\",job_name=bazel,status=SUCCESS,role=local")
                            .arg(format!("start_time=1,fetch_duration=200,build_duration=201,report_duration=202,total_duration={}",compile_time.unwrap_or("0".to_owned())))
                            .output()
                            .expect("failed to execute process");

                    println!("send_data: {:?}", send_data);
                    println!("status: {}", send_data.status);

                    // execute command to push to influxdb 

                    use fancy_regex::Regex;

                    let re = Regex::new(r"(?:Total)\s+(?:runtime)\s+(?<name3>((\d+\.?\d*)|(\.\d+)))((s|(ms)))\s(?<name4>((\d+\.?\d*)|(\.\d+)))?(%)*").unwrap();
                    let result = re.find("Total runtime 4s 43.16%");

                    assert!(result.is_ok(), "execution was successful");
                    let match_option = result.unwrap();

                    assert!(match_option.is_some(), "found a match");
                    let m = match_option.unwrap();

                    assert_eq!(m.start(), 0);
                    assert_eq!(m.end(), 23);
                    assert_eq!(m.as_str(), "Total runtime 4s 43.16%");

                },
                "parse_gradle_profile_to_grafana" => {
                    use regex::Regex as raw_regex;

                    let mut scan: scanner2<_, U64> = scanner2::scan_path2(extras).unwrap();

                    let single_line = scan.next_line().unwrap().unwrap();

                    println!("single_line {}", single_line);

                    // this force for minutes_seconds
                    let re = raw_regex::new(r"total_build_time(=)([0-9]*)").unwrap();
                    let caps = re.captures(&single_line).unwrap();
                    let total_build_time = caps.get(0).unwrap().as_str();

                    println!("total_build_time {}", total_build_time);

                    let splits = total_build_time.split("=").collect::<Vec<&str>>();
                    let total_build_time = splits[1].parse::<usize>().unwrap();
                    let total_build_time = total_build_time / 1000;

                    let send_data = Command::new("./influxdb_add.sh")
                            .arg("gradle")
                            .arg("host=\"sekarang\",job_name=bazel,status=SUCCESS,role=local")
                            .arg(format!("start_time=1,fetch_duration=200,build_duration=201,report_duration=202,total_duration={}",total_build_time))
                            .output()
                            .expect("failed to execute process");

                    println!("send_data: {:?}", send_data);
                    println!("status: {}", send_data.status);
                },
                _ => {
                    

                }
            }
        }
    } else {
        println! {"please supply -command=*value*"};
    }
}
