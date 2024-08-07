use clap::{arg, value_parser, Command, ArgAction};
use core::cell::Cell;
use html5ever::tendril::TendrilSink;

#[derive(Default, Debug)]
pub struct WaterDetail {
    pub is_number: String,
    pub st_code: String,
    pub ws_number: String 
}

impl WaterDetail {
    fn url(& self) -> minreq::URL {
        minreq::URL::from("https://dww2.tceq.texas.gov/DWW/JSP/WaterSystemDetail.jsp?tinwsys_is_number=".to_string() 
            + &self.is_number 
            + "&tinwsys_st_code=" 
            + &self.st_code 
            + "&wsnumber=" 
            + &self.ws_number 
            + "%20%20%20&DWWState="
            + &self.st_code)
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    // Make the default output file name: /current/env/path/[datetime]_out.csv
    let mut default_output_path: std::ffi::OsString = std::env::current_dir().unwrap().as_os_str().to_owned();
    let since_epoch: u64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    default_output_path.push("/".to_owned() + since_epoch.to_string().as_str() + "_out.csv");
    default_output_path = std::path::absolute(default_output_path).unwrap().as_os_str().to_owned();

    // Handle arguments
    let arg_matches = Command::new("tceq-scraper")
        .version("0.1")
        .about("Compiles water system data from https://dww2.tceq.texas.gov/ into a csv file.")
        .arg(
            arg!(-i <INPUT_CSV>)
                .value_parser(value_parser!(String))
                .id("input")
                .long("input")
                .required(true)
                .help("Provide a path to a csv file that contains TCEQ water detail info.")
                .long_help("CSV should consist of three columns:\n\ttinwsys_is_number\n\ttinwsys_st_code\n\twsnumber\nAll of these values can be found in the URL of the water detail page. (Example: https://dww2.tceq.texas.gov/DWW/JSP/WaterSystemDetail.jsp?tinwsys_is_number=5969&tinwsys_st_code=TX&wsnumber=TX2270001%20%20%20&DWWState=TX)")
                .action(ArgAction::Set)
        )
        .arg(
            arg!(-o <OUTPUT_CSV>)
                .value_parser(value_parser!(String))
                .id("output")
                .long("output")
                .required(false)
                .help("Choose a path to store water data.")
                .action(ArgAction::Set)
                .default_value(default_output_path)
        )
        .arg(
            arg!(-d <DELAY>)
                .value_parser(value_parser!(u32))
                .id("delay")
                .long("delay")
                .required(false)
                .help("Delay (milliseconds) between website requests.")
                .long_help("To avoid getting IP blocked for large requests, add a delay between each request to the website.")
                .action(ArgAction::Set)
                .default_value("3000")
        )
        .arg(
            arg!(-w <WS_NUMBER_HEADER>)
                .value_parser(value_parser!(String))
                .id("header_ws")
                .long("header_ws")
                .required(false)
                .help("Map the \"ws number\" header from the input file.")
                .long_help("In case the input file's \"ws number\" header does not go by the default name (\"ws_number\"), use this parameter to set a column from the input file as the \"ws number\" column using its header name.") 
                .action(ArgAction::Set)
                .default_value("ws_number")
        )
        .arg(
            arg!(-n <IS_NUMBER_HEADER>)
                .value_parser(value_parser!(String))
                .id("header_is")
                .long("header_is")
                .required(false)
                .help("Map the \"is number\" header from the input file.")
                .long_help("In case the input file's \"is number\" header does not go by the default name (\"is_number\"), use this parameter to set a column from the input file as the \"is number\" column using its header name.") 
                .action(ArgAction::Set)
                .default_value("is_number")
        )
        .arg(
            arg!(-s <STATE_CODE_HEADER>)
                .value_parser(value_parser!(String))
                .id("header_state")
                .long("header_state")
                .required(false)
                .help("Map the \"state code\" header from the input file.")
                .long_help("In case the input file's \"state code\" header does not go by the default name (\"st_code\"), use this parameter to set a column from the input file as the \"state code\" column using its header name.")
                .action(ArgAction::Set)
                .default_value("st_code")
        )
        .get_matches();
    
    let mut input_file_path: std::path::PathBuf = 
        std::fs::canonicalize(
            std::path::Path::new(
                arg_matches.get_one::<String>("input").expect("input file not provided.")
            )
        ).unwrap();
    let mut output_file_path: std::path::PathBuf = 
        std::path::absolute(
            std::path::Path::new(
                arg_matches.get_one::<String>("output").expect("output file is missing a default value.").as_str()
            )
        ).unwrap();
    
    // Verify that the input and output files are csv
    if input_file_path.as_path().extension().is_none() {
        input_file_path.set_extension(".csv");
    }
    else if input_file_path.as_path().extension().is_some_and(|ext| ext != "csv") {
        panic!("Input file is not a csv.");
    }

    if output_file_path.as_path().extension().is_none() {
        output_file_path.set_extension("csv");
    }
    else if output_file_path.as_path().extension().is_some_and(|ext| ext != "csv") {
        panic!("Output file is not a csv.");
    } 
    
    //println!("input: {} | output: {}", input_file_path.to_str().unwrap(), output_file_path.to_str().unwrap());

    // Map headers set in arguments to headers from input file
    println!("Reading headers from input...");
    let mut reader = csv::Reader::from_path(input_file_path).unwrap();
    let st_header_arg: &String = &arg_matches.get_one::<String>("header_state").expect("header_state is missing a default value.").to_string();
    let ws_header_arg: &String= &arg_matches.get_one::<String>("header_ws").expect("header_ws is missing a default value.").to_string();
    let is_header_arg: &String = &arg_matches.get_one::<String>("header_is").expect("header_is is missing a default value.").to_string();
    let mut header_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (idx, header) in reader.headers().expect("Header row missing from input file").iter().enumerate() {
        let h: String = header.to_string();
        if h == *st_header_arg {
            header_map.insert(st_header_arg.clone(), idx);
        }
        else if h == *ws_header_arg {
            header_map.insert(ws_header_arg.clone(), idx);
        }
        else if h == *is_header_arg {
            header_map.insert(is_header_arg.clone(), idx);
        }
        //println!("{:#?}", header);
    }
    
    // In case there are headers missing from the input,
    // show the user which headers are missing.
    if header_map.len() != 3 {
        let mut missing_headers_list: Vec<String> = Vec::new();
        if header_map.get(is_header_arg).is_none() {
            missing_headers_list.push(is_header_arg.clone());
        }
        if header_map.get(st_header_arg).is_none() {
            missing_headers_list.push(st_header_arg.clone());
        }
        if header_map.get(ws_header_arg).is_none() {
            missing_headers_list.push(ws_header_arg.clone());
        }
        let missing_headers: String = 
            missing_headers_list
                .iter_mut()
                .fold("".to_string(), |mut acc, h| {
                    if acc.len() > 0 {
                        acc.push_str(", ");
                    }
                    acc.push_str(h);
                    acc
                });
        panic!("Missing headers from input file: {}. Double check the header names that were supplied to the -w, -n, and -s arguments.", missing_headers);
    }
    println!("Headers successfully read.");

    // check if the file exists or has any problems opening
    // if so, print error and abort
    // otherwise, read the file into memory
    //let water_details: Vec<WaterDetail> = Vec::new();
    println!("Reading rows from input...");
    let water_details: Vec<WaterDetail> = 
        reader 
            .records()
            .map(|record| {
                WaterDetail {
                    is_number: record.as_ref().unwrap().get(*header_map.get(is_header_arg).unwrap()).unwrap().to_string(),
                    st_code: record.as_ref().unwrap().get(*header_map.get(st_header_arg).unwrap()).unwrap().to_string(),
                    ws_number: record.as_ref().unwrap().get(*header_map.get(ws_header_arg).unwrap()).unwrap().to_string()
                }
            })
            .collect();
    println!("Rows successfully read.");
    

    // Get HTML page of each water detail url
    let delay: u32 = *arg_matches.get_one::<u32>("delay").expect("output file is missing a default value.");
    println!("Sending requests for each water detail every {} milliseconds...", delay);
    let mut parse_opts = html5ever::driver::ParseOpts::default();
    parse_opts.tree_builder.scripting_enabled = false;
    for (idx, detail) in water_details.iter().enumerate() {
        // Debugging purposes
        //println!("{:#?}", detail);
        let url: minreq::URL = detail.url();
        match minreq::get(&url).send() {
            Ok(response) => {
                if response.status_code < 200 || response.status_code >= 300 {
                    println!("Failed to extract data because the response status was not OK. CSV Row number: {} | Status code: {} | Reason: {} | Url: {}", idx+1, response.status_code, response.reason_phrase, url)
                }
                else {
                    println!("Parsing URL (Row {})... ({})", idx+1, url);
                    //println!("Response: {}", response.as_str().unwrap());
                    //response.as_str().
                    //let stdin = io::stdin();
                    let dom = html5ever::parse_document(
                        markup5ever_rcdom::RcDom::default(),
                        parse_opts.clone()
                    )
                    .from_utf8()
                    //.read_from(&mut stdin.lock());
                    .read_from(&mut std::io::BufReader::new(response.as_bytes()))
                    .unwrap();

                    //dom.document.children
                    //.from_utf8()
                    //.read_from(response.as_bytes())
                    //walk(0, &dom.document);
                    let ignore_chars: &[char] = &['\t', '\n'];
                    let table_title = "Buyers of Water";
                    let table_text_node = find_by_text(&table_title, &dom.document, ignore_chars);
                    if table_text_node.is_some() {
                        let parent_table = parent_element(&table_text_node.unwrap());
                        if parent_table.is_some() {
                            let table = parent_table.expect("Unable to access parent table");
                            match table.data {
                                markup5ever_rcdom::NodeData::Element { ref name, ref attrs, ref template_contents, .. } => {
                                    println!("found parent table! name: {:?} | attrs: {:?} | contents: {:?}", name, attrs, template_contents);
                                },
                                _ => println!("parent table node type is not element")
                            }
                        } else {
                            println!("parent table not found");
                        }
                        
                        /*match &table_text_node.unwrap().data {
                            markup5ever_rcdom::NodeData::Text { ref contents } => {
                                let text_content = contents.borrow().escape_default().to_string();
                                println!("table text node contents: {}", text_content);
                            },
                            _ => println!("The wrong node type was returned")
                        }*/
                    } else {
                        println!("{} table text node is None.", table_title);
                    }
                }
            },
            Err(e) => println!("Failed to extract data because the request was unsuccessful. CSV Row number: {} | Error: {}", idx+1, e)
        }
    }

    // Get tecq water data page

}

fn parent_element(node: &markup5ever_rcdom::Handle) -> Option<markup5ever_rcdom::Handle> {
    let mut p: Option<markup5ever_rcdom::Handle> = Some(node.clone());
    while p.is_some() {
        let p_handle = p.expect("Parent node is missing.");
        match p_handle.data {
            markup5ever_rcdom::NodeData::Element { .. } => {
                return Some(p_handle);
            },
            _ => { }
        }
        match p_handle.parent.take() {
            Some(p_weak) => {
                p = p_weak.upgrade();
            },
            None => return None
        }
    }
    return None
}

fn find_by_text(table_name: &str, node: &markup5ever_rcdom::Handle, ignore_chars: &[char]) -> Option<markup5ever_rcdom::Handle> {
    match node.data {
        markup5ever_rcdom::NodeData::Text { ref contents } => {
            let text_content = contents.borrow().to_string().trim().replace(ignore_chars, "");
            println!("text content: {}", text_content);
            if text_content.eq(table_name) {
                println!("Found table: {}", text_content);
                return Some(node.clone())
            }
            //println!("#text: {}", contents.borrow().escape_default())
        },
        /*markup5ever_rcdom::NodeData::Element { ref template_contents, .. } => {
            let text_content = template_contents.borrow().unwrap().data.;
            if text_content.eq(table_name) {
                println!("Found table: {}", text_content);
                return Some(node.clone())
            }
        },*/
        _ => { }
    }
    
    for child in node.children.borrow().iter() {
        let h = find_by_text(table_name, child, ignore_chars);
        if h.is_some() {
            return h
        }
    }

    return None

    /*for child in node.children.borrow().iter() {
        walk(indent + 4, child);
    }*/
}

/*fn walk(indent: usize, handle: &markup5ever_rcdom::Handle) {
    let node = handle;
    for _ in 0..indent {
        print!(" ");
    }
    match node.data {
        markup5ever_rcdom::NodeData::Document => println!("#Document"),

        markup5ever_rcdom::NodeData::Doctype {
            ref name,
            ref public_id,
            ref system_id,
        } => println!("<!DOCTYPE {} \"{}\" \"{}\">", name, public_id, system_id),

        markup5ever_rcdom::NodeData::Text { ref contents } => {
            println!("#text: {}", contents.borrow().escape_default())
        },

        markup5ever_rcdom::NodeData::Comment { ref contents } => println!("<!-- {} -->", contents.escape_default()),

        markup5ever_rcdom::NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            //assert!(name.ns == ns!(html));
            print!("<{}", name.local);
            for attr in attrs.borrow().iter() {
                //assert!(attr.name.ns == ns!());
                print!(" {}=\"{}\"", attr.name.local, attr.value);
            }
            println!(">");
        },

        markup5ever_rcdom::NodeData::ProcessingInstruction { .. } => unreachable!(),
    }

    for child in node.children.borrow().iter() {
        walk(indent + 4, child);
    }
}*/
