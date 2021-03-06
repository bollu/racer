#![feature(collections, core, old_io, old_path, rustc_private, env, test)]

#[macro_use]
extern crate log;
extern crate syntax;
extern crate collections;
extern crate core;
extern crate libc;

//use std::c_str::CString;
use libc::c_char;
//for c string
use std::ffi;
//for channels to send data around
use std::sync::mpsc::channel;
//for recieving data
use std::sync::mpsc;
//for spawn
use std::thread;

#[cfg(not(test))]
use racer::Match;
#[cfg(not(test))]
use racer::util::getline;
#[cfg(not(test))]
use racer::nameres::{do_file_search, do_external_search};
#[cfg(not(test))]
use racer::scopes;

pub mod racer;

#[cfg(not(test))]
fn match_with_snippet_fn(m:Match) {
    let (linenum, charnum) = scopes::point_to_coords_from_file(&m.filepath, m.point).unwrap();
    if m.matchstr == "" {
        panic!("MATCHSTR is empty - waddup?");
    }

    let snippet = racer::snippets::snippet_for_match(&m);
    println!("MATCH {};{};{};{};{};{:?};{}", m.matchstr,
             snippet,
             linenum.to_string(),
             charnum.to_string(),
             m.filepath.as_str().unwrap(),
             m.mtype,
             m.contextstr,
             );
}

#[cfg(not(test))]
fn match_fn(m:Match) {
    let (linenum, charnum) = scopes::point_to_coords_from_file(&m.filepath, m.point).unwrap();
    if m.matchstr == "" {
        panic!("MATCHSTR is empty - waddup?");
    }
    println!("MATCH {},{},{},{},{:?},{}", m.matchstr,
             linenum.to_string(),
             charnum.to_string(),
             m.filepath.as_str().unwrap(),
             m.mtype,
             m.contextstr
             );
}

#[cfg(not(test))]
fn complete(match_found : &Fn(Match)) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Provide more arguments!");
        print_usage();
        std::env::set_exit_status(1);
        return;
    }
    match args[2].parse::<usize>() {
        Ok(linenum) => {
            // input: linenum, colnum, fname
            if args.len() < 5 {
                println!("Provide more arguments!");
                print_usage();
                std::env::set_exit_status(1);
                return;
            }
            let charnum = args[3].parse::<usize>().unwrap();
            let fname = &args[4][];
            let fpath = Path::new(fname);
            let src = racer::load_file(&fpath);
            let line = &*getline(&fpath, linenum);
            let (start, pos) = racer::util::expand_ident(line, charnum);
            println!("PREFIX {},{},{}", start, pos, &line[start..pos]);

            let point = scopes::coords_to_point(&*src, linenum, charnum);
            for m in racer::complete_from_file(&*src, &fpath, point) {
                match_found(m);
            }
        }
        Err(_) => {
            // input: a command line string passed in
            let arg = &args[2][];
            let it = arg.split_str("::");
            let p : Vec<&str> = it.collect();

            for m in do_file_search(p[0], &Path::new(".")) {
                if p.len() == 1 {
                    match_found(m);
                } else {
                    for m in do_external_search(&p[1..], &m.filepath, m.point, racer::SearchType::StartsWith, racer::Namespace::BothNamespaces) {
                        match_found(m);
                    }
                }
            }
        }
    }
}

#[cfg(not(test))]
fn prefix() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        println!("Provide more arguments!");
        print_usage();
        std::env::set_exit_status(1);
        return;
    }
    let linenum = args[2].parse::<usize>().unwrap();
    let charnum = args[3].parse::<usize>().unwrap();
    let fname = &args[4][];

    // print the start, end, and the identifier prefix being matched
    let path = Path::new(fname);
    let line = &*getline(&path, linenum);
    let (start, pos) = racer::util::expand_ident(line, charnum);
    println!("PREFIX {},{},{}", start, pos, &line[start..pos]);
}

#[cfg(not(test))]
fn find_definition() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        println!("Provide more arguments!");
        print_usage();
        std::env::set_exit_status(1);
        return;
    }
    let linenum = args[2].parse::<usize>().unwrap();
    let charnum = args[3].parse::<usize>().unwrap();
    let fname = &args[4][];
    let fpath = Path::new(fname);
    let src = racer::load_file(&fpath);
    let pos = scopes::coords_to_point(&*src, linenum, charnum);

    racer::find_definition(&*src, &fpath, pos).map(match_fn);
}

#[cfg(not(test))]
fn print_usage() {
    let program = std::env::args().next().unwrap().clone();
    println!("usage: {} complete linenum charnum fname", program);
    println!("or:    {} find-definition linenum charnum fname", program);
    println!("or:    {} complete fullyqualifiedname   (e.g. std::io::)",program);
    println!("or:    {} prefix linenum charnum fname",program);
    println!("or replace complete with complete-with-snippet for more detailed completions.");
}


#[cfg(not(test))]
fn main() {
    if std::env::var("RUST_SRC_PATH").is_err() {
        let default_env_path = "/home/bollu/prog/rust/src";
        //print!("RUST_SRC_PATH is not set. setting to {}", default_env_path);
        std::env::set_var("RUST_SRC_PATH", default_env_path);
        //println!("RUST_SRC_PATH environment variable must be set");
        //std::env::set_exit_status(1);
        //return;
    }

    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        print_usage();
        std::env::set_exit_status(1);
        return;
    }

    let command = &args[1][];
    match command {
        "prefix" => prefix(),
        "complete" => complete(&match_fn),
        "complete-with-snippet" => complete(&match_with_snippet_fn),
        "find-definition" => find_definition(),
        "help" => print_usage(),
        cmd => {
            println!("Sorry, I didn't understand command {}", cmd);
            print_usage();
            std::env::set_exit_status(1);
            return;
        }
    }
}


fn gen_match_string_for_snippet(m : Match) -> String {
    let (linenum, charnum) = match scopes::point_to_coords_from_file(&m.filepath, m.point) {
        Some(point) => point,
        None => return String::from_str("PANIC: no point found")
    };
    if m.matchstr == "" {
        return String::from_str("PANIC: MATCHSTR is empty")
    }

    let snippet = racer::snippets::snippet_for_match(&m);
    let match_string = format!("MATCH {};{};{};{};{};{:?};{}", m.matchstr,
                               snippet,
                               linenum.to_string(),
                               charnum.to_string(),
                               m.filepath.as_str().unwrap(),
                               m.mtype,
                               m.contextstr,
                               );
    match_string
}



fn gen_match_str_for_fn_defn(m : Match) -> String { 
    let (linenum, charnum) = scopes::point_to_coords_from_file(&m.filepath, m.point).unwrap();
    if m.matchstr == "" {
        panic!("MATCHSTR is empty - waddup?");
    }

    let match_string = format!("MATCH {},{},{},{},{:?},{}", m.matchstr,
             linenum.to_string(),
             charnum.to_string(),
             m.filepath.as_str().unwrap(),
             m.mtype,
             m.contextstr
             );

    match_string

}

#[no_mangle]
pub extern "C" fn complete_with_snippet_ffi_may_panic(linenum : usize, charnum : usize, fpath: Path, tx: mpsc::Sender<String>) {
    

    let src = racer::load_file(&fpath);
    let line = &*getline(&fpath, linenum);
    let (start, pos) = racer::util::expand_ident(line, charnum);        
    let point = scopes::coords_to_point(&*src, linenum, charnum);

    //HACK: this can panic
    let iter = racer::complete_from_file(&*src, &fpath, point);

    let mut output_string = String::new();    
    for m in iter {
        let mut match_string = gen_match_string_for_snippet(m);
        output_string.push_str(match_string.as_slice());
        output_string.push_str("\n");
        
    }

    tx.send(output_string);
//    let output_c_str = ffi::CString::from_slice(output_string.as_bytes());
//    unsafe { libc::strcpy(out_raw, output_c_str.as_ptr()); }
}



#[no_mangle]
pub extern "C" fn complete_with_snippet_ffi(linenum : usize, charnum : usize, fname_raw: *const c_char, out_raw: *mut c_char) {
    //null out the out string just in case
    {
        let null_string = ffi::CString::from_slice("PANICD BITCH".as_bytes());
        unsafe { libc::strcpy(out_raw, null_string.as_ptr()); }
    }

   let (tx, rx) = channel();

   let fpath =  {
        let fname_bytes = unsafe { ffi::c_str_to_bytes(&fname_raw) };
        let fname = std::str::from_utf8(fname_bytes).ok().unwrap();  
        Path::new(fname)
    };


    thread::spawn(move || { complete_with_snippet_ffi_may_panic(linenum, charnum, fpath, tx) });


    match rx.recv() {
        Ok(output_string) => {
            let output_c_str = ffi::CString::from_slice(output_string.as_bytes());
            unsafe { libc::strcpy(out_raw, output_c_str.as_ptr()); }

        }

        Err(_) => {}
    }
    
}

pub extern "C" fn find_definition_ffi_may_panic(linenum : usize, charnum : usize, fname_raw: *const c_char) {
    let fpath =  {
        let fname_bytes = unsafe { ffi::c_str_to_bytes(&fname_raw) };
        let fname = std::str::from_utf8(fname_bytes).ok().unwrap();  
        Path::new(fname)
    };
    let src = racer::load_file(&fpath);
    let pos = scopes::coords_to_point(&*src, linenum, charnum);

    let opt_defn = racer::find_definition(&*src, &fpath, pos);

    match opt_defn {
        Some(defn) => {
            let mut match_string = gen_match_str_for_fn_defn(defn);
            let mut output_string = String::new();    
    
            output_string.push_str(match_string.as_slice());
            output_string.push_str("\n");
            
            //let output_c_str = ffi::CString::from_slice(output_string.as_bytes());
           //unsafe { libc::strcpy(out_raw, output_c_str.as_ptr()); }
        }

        None => {}
    }
}
    
