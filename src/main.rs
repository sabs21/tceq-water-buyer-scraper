use clap::{arg, value_parser, Command, ArgAction};

#[derive(Default, Debug)]
pub struct WaterDetail {
    pub is_number: String,
    pub st_code: String,
    pub ws_number: String 
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

    // check if the file exists or has any problems opening
    // if so, print error and abort
    // otherwise, read the file into memory
    //let water_details: Vec<WaterDetail> = Vec::new();
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
    for detail in water_details.iter() {
        println!("{:#?}", detail);
    }
}
