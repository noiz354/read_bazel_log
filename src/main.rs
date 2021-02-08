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

use std::vec::Vec;
// use vec_three_item as Vec<(String, String, String)>;

fn main() {
    let args: Vec<String> = env::args().collect();

    // println!("{:?}", args);
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
            }else if command.contains("result-manifest"){
                let v: Vec<&str> = command.split('=').skip(1).collect();
                if v.len() == 1 {
                    let v: Vec<&str> = v[0].split(',').collect();
                    command_parse = Some(v);
                }
            }
        }
        println!("result-manifest \n\n\n{:?}\n\n\n", command_parse);



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
            }else if command.contains("main-manifest") {
                let v: Vec<&str> = command.split('=').skip(1).collect();
                if v.len() == 1 {
                    let v: Vec<&str> = v[0].split(',').collect();
                    extras_command_parse = Some(v);
                }
            }
        }

        println!("main-manifest \n\n\n{:?}\n\n\n", extras_command_parse);

        let mut third_command_parse : Option<Vec<&str>> = None;
        // find command method
        if let Some(command) = args.get(3) {
            // find `=`
            if command.contains("child-manifest") {
                let v: Vec<&str> = command.split('=').skip(1).collect();
                if v.len() == 1 {
                    let v: Vec<&str> = v[0].split(',').collect();
                    third_command_parse = Some(v);
                }
            }
        }

        println!("child-manifest \n\n\n{:?}\n\n\n", extras_command_parse);

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
                "android_manifest_merger" => {
                    use quick_xml::{Reader, Writer};
                    use quick_xml::events::{Event, BytesEnd, BytesStart};
                    use std::fs::OpenOptions;

                    // transverse the dependencies

                    // read xml here
                    
                    let xml = r#"<this_tag k1="v1" k2="v2"><child>text</child></this_tag>"#;
                    let mut reader = Reader::from_str(xml);
                    reader.trim_text(true);
                    
                    let file_options = OpenOptions::new()
                    .append(true)
                    .read(true)
                    .open("foo.txt");
                    let file = file_options.unwrap();

                    let mut writer = Writer::new(file);
                    let mut buf = Vec::new();
                    loop {
                        match reader.read_event(&mut buf) {
                            Ok(Event::Start(ref e)) if e.name() == b"this_tag" => {
                    
                                // crates a new element ... alternatively we could reuse `e` by calling
                                // `e.into_owned()`
                                let mut elem = BytesStart::owned(b"my_elem".to_vec(), "my_elem".len());
                    
                                // collect existing attributes
                                elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));
                    
                                // copy existing attributes, adds a new my-key="some value" attribute
                                elem.push_attribute(("my-key", "some value"));
                    
                                // writes the event to the writer
                                let _ = writer.write("\n".as_bytes());
                                let write_event = writer.write_event(Event::Start(elem));
                                println!("{:?}", write_event);
                                assert!(write_event.is_ok());
                            },
                            Ok(Event::End(ref e)) if e.name() == b"this_tag" => {
                                assert!(writer.write_event(Event::End(BytesEnd::borrowed(b"my_elem"))).is_ok());
                            },
                            Ok(Event::Eof) => break,
                            // we can either move or borrow the event to write, depending on your use-case
                            Ok(e) => assert!(writer.write_event(&e).is_ok()),
                            Err(e) => panic!("{}", e),
                        }
                        buf.clear();
                    }
                    
                    // compare as bytes is 
                    // let result = writer.into_inner();//.into_inner();
                    // let expected = r#"<my_elem k1="v1" k2="v2" my-key="some value"><child>text</child></my_elem>"#;
                    // assert_eq!(result.bytes(), expected.as_bytes());
                },
                _ => {
                    

                }
            }
        }

        // this must be parse AndroidManifest xml
        if third_command_parse.is_some() {
            // hard code the path 
            let suffix_path = r"/Users/normansyahputa/Documents/dev/repo/BazelApplication/";

            let result_manifest = command_parse;
            let main_manifest = extras_command_parse.clone();
            let child_manifest = third_command_parse.unwrap();

            println!("masuk sini\n\n\n{:?}", result_manifest);
            println!("masuk sini\n\n\n{:?}", main_manifest);
            println!("masuk sini\n\n\n{:?}", child_manifest);
            let mut hash_set = HashSet::new();

            for var in child_manifest {
                let mut suffix_path = String::from(suffix_path);
                suffix_path.push_str(var);

                // println!("process file : {}", suffix_path);

                if let Ok(mut file) = read_and_write_file(suffix_path.as_str()){
                    match gather_uses_permission(&mut file){
                        Ok(e) => {
                            for e_item in e.to_owned(){
                                if !hash_set.contains(&e_item){
                                    hash_set.insert(e_item);
                                }
                            }
                        },
                        Err(print_message) => {
                            println!("{}", print_message);
                        },
                    }
                }else{
                    println!("cannot read file {}", suffix_path);
                }
            }

            println!("masuk sini hash_set {:?}", hash_set);

            let (final_path_real,_) = result_manifest[0];
            let mut final_path = String::from(suffix_path);
            final_path.push_str("/");
            final_path.push_str(final_path_real);
            println!("final_path {}", final_path);  

            let var = main_manifest[0];
            let mut suffix_path = String::from(suffix_path);
            suffix_path.push_str(var);
            match read_and_write_file(suffix_path.as_str()){
                Ok(mut file) => {
                    let content = parse_main_manifest(&mut file, &hash_set);
                    let mut file = create_read_and_write_file(final_path.as_str()).unwrap();
                    file.write_all(content.as_bytes()).expect("unable to write");
                    let _ = file.flush().unwrap();
                }
                Err(e) => {
                    println!("cannot read file {} : {:?}", suffix_path, e);
                }
            }
    
        }
    } else {
        println! {"please supply -command=*value*"};
    }
}

use std::collections::HashSet;
use std::io::*;
use std::fs::{OpenOptions, File};
fn create_read_and_write_file(path : &str) -> Result<File>{
    let file = OpenOptions::new().write(true)
    .create_new(true)
    .read(true)
    .append(true)
    .open(path)?;

    Ok(file)
}

fn read_and_write_file(path : &str) -> Result<File>{
    let file = OpenOptions::new()
            .read(true)
            .open(path)?;
    Ok(file)
}

use quick_xml::{Reader, Writer};
use quick_xml::events::{Event, BytesEnd, BytesStart};
use std::str::from_utf8;
use quick_xml::events::Event::*;

fn parse_main_manifest(file: &mut File, hash_set: &HashSet<(String, String, String)> ) -> String {
    let mut string = String::with_capacity(initial_buffer_size(&file));
    file.read_to_string(&mut string).unwrap();
    let mut reader = Reader::from_str(string.as_str());
    reader.trim_text(true);

    let find_index = string.find("    <application").unwrap();

    for (event, ns_value, ns) in hash_set.iter(){
        string.insert_str(find_index, format!(" <{} {}=\"{}\" />\n", event, ns, ns_value).as_str());
    }   

    println!("{} {}", find_index, string);

    string

}

fn initial_buffer_size(file: &File) -> usize {
    // Allocate one extra byte so the buffer doesn't need to grow before the
    // final `read` call at the end of the file.  Don't worry about `usize`
    // overflow because reading will fail regardless in that case.
    file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0)
}

fn gather_uses_permission(file: &mut File) -> std::result::Result<Vec<(String, String, String)>, &'static str>{
    let mut string = String::with_capacity(initial_buffer_size(&file));
    file.read_to_string(&mut string).unwrap();
    
    let mut reader = Reader::from_str(string.as_str());
    reader.trim_text(true);
    let mut writer = Writer::new(file);
    

    // println!("masuk sini {:?}", string);
    let mut uses_permissions = Vec::new();
    let mut buf_namespace = Vec::new();
    let mut ns_buf_namespace = Vec::new();
    let mut count = 0;
    // let mut txt = Vec::new();
    loop {
        let event = reader.read_namespaced_event(&mut buf_namespace, &mut ns_buf_namespace);
        // println!("\nevent start {:?}", event);
        let event = match event {
            Ok((Some(b"http://schemas.android.com/apk/res/android"), Start(event))) => {
                // println!("Some, Start {:?}", event);
                event
            }, 
            Ok((None, Decl(_))) => continue,
            Ok((None, Start(event))) => {
                // println!("None, Start start {:?}", String::from_utf8(event.name().to_vec()).unwrap_or(String::from("")));
                let e_ = event
                    .attributes()
                    .map(|a| {
                        let a = a.unwrap();
                        (a.value, a.key)
                    })
                    .map(|(a, b)| (a.into_owned(), b))
                    .map(|(a, b)| {
                        (
                            String::from_utf8(a).unwrap_or(String::from("")),
                            String::from_utf8(b.to_vec()).unwrap_or(String::from("")),
                        )
                    })
                    .collect::<Vec<(String, String)>>();
                // println!("None, Start exit {:?}", e_);
                event
            }
            Ok((None, Empty(e))) => {
                // println!("None, Empty start {:?}", e);
                let mut e_ = e
                    .attributes()
                    .map(|a| {
                        let a = a.unwrap();
                        (a.value, a.key)
                    })
                    .map(|(a, b)| (a.into_owned(), b))
                    .map(|(a, b)| {
                        (
                            String::from_utf8(e.name().to_vec()).unwrap_or(String::from("")),
                            String::from_utf8(a).unwrap_or(String::from("")),
                            String::from_utf8(b.to_vec()).unwrap_or(String::from("")),
                        )
                    })
                    .collect::<Vec<(String, String, String)>>();
                if e.name() == b"uses-permission" {
                    uses_permissions.append(&mut e_);
                }
                // println!("None, Empty exit {:?}", e_);
                e
            }
            Ok((None, End(e))) => {
                // println!("None, End {:?}", e);
                continue
            },
            Ok((None, Comment(_))) => {
                continue
            },
            Ok((Some(_), Start(_))) => panic!("expecting namespace to resolve to 'www1'"),
            Ok((_, Event::Eof)) => break,
            _ => panic!("expecting namespace resolution"),
        };

        // println!("event exit {:?}\n", event);
    }
    buf_namespace.clear();

    if uses_permissions.len() == 0 {
        return Err("empty uses permission");
    }else{
        return Ok(uses_permissions);
    }
    

    // let mut buf = Vec::new();
    // println!("masuk sini {:?}", txt);
    // loop {
    //     match reader.read_event(&mut buf) {
    //         Ok(Event::Start(ref e)) if e.name() == b"manifest" => {
    //             // fn e_type(e_: 'static Vec<'static[u8]>){}
    //             let e_ = e.attributes()
    //                     .map(|a| a.unwrap().value)
    //                     .map(|a| a.into_owned() )
    //                     .map(|a| String::from_utf8(a))
    //                     .collect::<Vec<_>>();

    //             println!("masuk sini 2 {:?}", e_);
    
    //             // crates a new element ... alternatively we could reuse `e` by calling
    //             // `e.into_owned()`
    //             let mut elem = BytesStart::owned(b"my_elem".to_vec(), "my_elem".len());
    
    //             // collect existing attributes
    //             elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));
    
    //             // copy existing attributes, adds a new my-key="some value" attribute
    //             elem.push_attribute(("my-key", "some value"));
    
    //             // writes the event to the writer
    //             let _ = writer.write("\n".as_bytes());
    //             let write_event = writer.write_event(Event::Start(elem));
    //             println!("{:?}", write_event);
    //             assert!(write_event.is_ok());
    //         },
    //         Ok(Event::End(ref e)) if e.name() == b"this_tag" => {
    //             assert!(writer.write_event(Event::End(BytesEnd::borrowed(b"my_elem"))).is_ok());
    //         },
    //         Ok(Event::Eof) => break,
    //         // we can either move or borrow the event to write, depending on your use-case
    //         Ok(e) => assert!(writer.write_event(&e).is_ok()),
    //         Err(e) => panic!("{}", e),
    //     }
    //     buf.clear();
    // }
}

fn add(x:usize, y:usize) -> usize{
    x+y
}

fn bad_add(x:usize, y:usize) -> usize{
    x+y
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_gather_permission(){
        let var = "AndroidManifest.xml";
        let suffix_path = "./";
        // let suffix_path  = r"/Users/normansyahputa/Documents/dev/repo/quick-xml/";

        let mut suffix_path_str = String::from(suffix_path);
        suffix_path_str.push_str(var);
        if let Ok(mut file) = read_and_write_file(suffix_path_str.as_str()){
            gather_uses_permission(&mut file);
        }else{
            println!("error nih bro");
        }

        if let Err(e) = read_and_write_file(suffix_path_str.as_str()){
            println!("{} \nerror nih bro {:?}", suffix_path_str, e);
        }
        
    }

    #[test]
    fn test_main_permission(){
        let var = "AndroidManifest.xml";
        let suffix_path = "/Users/normansyahputa/Documents/dev/repo/BazelApplication/app/src/main/";
        // let suffix_path  = r"/Users/normansyahputa/Documents/dev/repo/quick-xml/";


        let mut hash_set = HashSet::new();
        hash_set.insert((
            "uses-permission".to_string(), 
            "android.permission.ACCESS_NETWORK_STATE".to_string(),
            "android:name".to_string(), 
        ));

        let mut suffix_path_str = String::from(suffix_path);
        suffix_path_str.push_str(var);

        let final_path = "./AndroidManifest_result.xml";

        if let Ok(mut file) = read_and_write_file(suffix_path_str.as_str()){
            let content = parse_main_manifest(&mut file, &hash_set);
            let mut file = create_read_and_write_file(final_path).unwrap();
            file.write_all(content.as_bytes()).expect("unable to write");
            
        }else{
            println!("error nih bro");
        }

        if let Err(e) = read_and_write_file(suffix_path_str.as_str()){
            println!("{} \nerror nih bro {:?}", suffix_path_str, e);
        }
        
    }

    #[test]
    fn test_read_and_write_file(){
        use std::io::Error;
        use std::io::ErrorKind::AlreadyExists;

        let mut file = match create_read_and_write_file("/Users/normansyahputa/Documents/dev/repo/read_bazel_log/foo.txt"){
            Ok(file) => {file},
            Err(ref e) => {
                let mut file = None;
                if e.kind() == AlreadyExists {
                    file = Some(read_and_write_file("/Users/normansyahputa/Documents/dev/repo/read_bazel_log/foo.txt").unwrap());
                }
                file.unwrap()
            },
        };

        let mut string = String::with_capacity(initial_buffer_size(&file));
        file.read_to_string(&mut string).unwrap();
        println!("{}","masuk sini gak ya?");
        println!("{}",string);
        assert_eq!("", string);
    }

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }

    #[test]
    fn test_bad_add() {
        // This assert would fire and test will fail.
        // Please note, that private functions can be tested too!
        assert_eq!(bad_add(1, 2), 3);
    }
}
