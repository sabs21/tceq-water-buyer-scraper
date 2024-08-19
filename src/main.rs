use clap::{arg, value_parser, Command, ArgAction};

// Includes necessary sql queries into the shipped exe
static INSERT_WATER_DETAIL_SQL: &'static str = include_str!("../src/queries/insert_water_detail.sql");
static INSERT_BUYER_SELLER_RELATIONSHIP_SQL: &'static str = include_str!("../src/queries/insert_buyer_seller_relationship.sql");

#[derive(Default, Debug)]
pub struct BuyerSellerRelationship {
    pub buyer: String,
    pub buyer_name: String,
    pub seller: String,
    pub population: String,
    pub availability: String
}

#[derive(Default, Debug, Clone)]
pub struct WaterDetail {
    pub is_number: Option<String>,
    pub st_code: String,
    pub ws_number: String,
    pub name: Option<String>
}

impl WaterDetail {
    fn url(& self) -> minreq::URL {
        minreq::URL::from("https://dww2.tceq.texas.gov/DWW/JSP/WaterSystemDetail.jsp?tinwsys_is_number=".to_string() 
            + &self.is_number.clone().expect("Missing is_number. Cannot build URL.")
            + "&tinwsys_st_code=" 
            + &self.st_code 
            + "&wsnumber=" 
            + &self.ws_number 
            + "%20%20%20&DWWState="
            + &self.st_code)
    }
}

fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    
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
    let mut input_water_details: Vec<WaterDetail> = 
        reader 
            .records()
            .map(|record| {
                WaterDetail {
                    is_number: Some(record.as_ref().unwrap().get(*header_map.get(is_header_arg).unwrap()).unwrap().to_string()),
                    st_code: record.as_ref().unwrap().get(*header_map.get(st_header_arg).unwrap()).unwrap().to_string(),
                    ws_number: record.as_ref().unwrap().get(*header_map.get(ws_header_arg).unwrap()).unwrap().to_string(),
                    name: None // Name gets scraped from the page
                }
            })
            .collect();
    println!("Rows successfully read.");

    // Get HTML page of each water detail url
    let delay: u32 = *arg_matches.get_one::<u32>("delay").expect("output file is missing a default value.");
    println!("Sending requests for each water detail every {} milliseconds...", delay);
    let whitespace_regex = regex::Regex::new(r"\s+").unwrap();
    for (idx, detail) in input_water_details.iter_mut().enumerate() {
        // Debugging purposes
        //println!("{:#?}", detail);
        println!("Scraping water detail {}...", detail.ws_number);
        let url: minreq::URL = detail.url();
        match minreq::get(&url).send() {
            Ok(response) => {
                if response.status_code < 200 || response.status_code >= 300 {
                    println!("Failed to extract data because the response status was not OK. CSV Row number: {} | Status code: {} | Reason: {} | Url: {}", idx+1, response.status_code, response.reason_phrase, url)
                }
                else {
                    println!("Parsing URL (Row {})... ({})", idx+1, url);
                    // Get tecq water data page
                    let dom = scraper::Html::parse_document(response.as_str().expect("Failed to parse webpage."));
                    // Fetch the name of this water detail
                    if detail.name.is_none() {
                        if let Some(info_table) = get_table_by_name(&"Water System Detail Information".to_string(), &dom) {
                            detail.name = get_value_from_header(&"Water System Name:".to_string(), &info_table);
                        }
                    }
                    // The key for the hash map is the water detail number string
                    let mut parsed_water_details: std::collections::HashMap<String, WaterDetail> = std::collections::HashMap::new();
                    let root_water_detail = WaterDetail {
                        ws_number: detail.ws_number.clone(),
                        is_number: detail.is_number.clone(),
                        st_code: detail.st_code.clone(),
                        name: detail.name.clone()
                    };
                    parsed_water_details.insert(detail.name.clone().unwrap(), root_water_detail.clone());
                    if insert_water_detail(&root_water_detail).is_err() {
                        println!("Failed to write water detail {} to database.", root_water_detail.ws_number);
                    }
                    if let Some(wbt) = get_table_by_name(&"Buyers of Water".to_string(), &dom) {
                        let row_selector = scraper::Selector::parse("tbody tr td").expect("Unable to find table rows");
                        //println!("Found buyers of water table!");
                        let column_delimiter_regex = regex::Regex::new(r" - |sells to|\/").unwrap();
                        let rows = 
                            wbt
                                .select(&row_selector)
                                .collect::<Vec<scraper::ElementRef>>();
                        let mut relationships: Vec<BuyerSellerRelationship> = Vec::new();
                        //let mut water_details: Vec<WaterDetail> = vec![];
                        for row in rows {
                            // Deserialize raw relationship text
                            // The order of the relationship data is as follows:
                            // 1. Seller's Water System ID
                            // 2. Name of Buyer
                            // 3. Buyer's Water System ID
                            // 4. Population
                            // 5. Availability (can be blank)
                            let mut row_data: Vec<String> = Vec::new();
                            for txt in row.text().filter(|t| !t.trim().is_empty()) {
                                let relationship_text = whitespace_regex.replace_all(txt, " ");
                                if column_delimiter_regex.is_match(&relationship_text) {
                                    for m in column_delimiter_regex.split(&relationship_text).filter(|res| !res.trim().is_empty()) {
                                        row_data.push(m.trim().to_string());
                                    }
                                }
                                else {
                                    row_data.push(relationship_text.trim().to_string());
                                }
                            }
                            // In case availability is left blank, we must add 
                            // an empty string to row data so that the length is 5.
                            if row_data.len() != 0 {
                                while row_data.len() < 5 {
                                    row_data.push("".to_string());
                                }
                                relationships.push(BuyerSellerRelationship {
                                    seller: row_data[0].clone(),
                                    buyer_name: row_data[1].clone(),
                                    buyer: row_data[2].clone(),
                                    population: row_data[3].clone(),
                                    availability: row_data[4].clone()
                                });
                            }
                        }
                        for r in relationships.iter() {
                            //println!("row: {}", r_idx);
                            //println!("{:#?}", r);
                            if parsed_water_details.get(&r.buyer.clone()).is_none() {
                                let wd = WaterDetail {
                                    ws_number: r.buyer.clone(),
                                    st_code: r.buyer[..2].to_string(),
                                    name: Some(r.buyer_name.clone()),
                                    is_number: None
                                };
                                //println!("{:#?}", wd);
                                parsed_water_details.insert(wd.ws_number.clone(), wd.clone());
                                // Insert new water details into database
                                if insert_water_detail(&wd).is_ok() {
                                    println!("Added water detail {} to database.", wd.ws_number);
                                }
                                else {
                                    println!("Skipped water detail {} because it already exists in database.", wd.ws_number);
                                }
                            }
                        }

                        // Insert new buyer/seller relationships into database
                        for r in relationships.iter() {
                            //println!("row: {}", r_idx);
                            //println!("{:#?}", r);
                            if insert_buyer_seller_relationship(r).is_ok() {
                                println!("Added relationship '{} sells to {}' to database.", r.buyer, r.seller);
                            }
                            else {
                                println!("Skipped relationship '{} sells to {}' because it already exists in database.", r.buyer, r.seller);
                            }
                        }

                        println!("Finished scraping {}.", detail.ws_number);
                    }
                }
            },
            Err(e) => println!("Failed to extract data because the request was unsuccessful. CSV Row number: {} | Error: {}", idx+1, e)
        }
    }
}

fn get_table_by_name<'a>(name: &'a String, dom: &'a scraper::Html) -> Option<scraper::ElementRef<'a>> {
    let table_selector = scraper::Selector::parse("body table tbody tr td table").expect("Unable to find a table within the webpage.");
    return dom
            .select(&table_selector)
            .filter(|el| {
                //if let Some(header) = el.select(table_header_selector)
                let mut text_iter = el.text().filter(|t| !t.trim().is_empty());
                if let Some(first_header_text) = text_iter.next() {
                    let txt = first_header_text.trim();
                    return txt == name
                }
                return false
            })
            .collect::<Vec<scraper::ElementRef>>()
            .first()
            .copied()
}

// Finds a header (the key), then returns the value
// NOTE: if the header in TCEQ includes a colon (i.e., "Water System Name:"), 
// then header_name needs that colon too.
fn get_value_from_header(header_name: &String, table: &scraper::ElementRef) -> Option<String> {
    let whitespace_regex = regex::Regex::new(r"\s+").unwrap();
    let cell_header_text_selector = scraper::Selector::parse("tbody tr td").expect("Unable to find header text");
    let mut found_header: bool = false;
    let cells = table.select(&cell_header_text_selector);
    for cell in cells {
        for raw_txt in cell.text().filter(|t| !t.trim().is_empty()) {
            let txt = whitespace_regex.replace_all(raw_txt.trim(), " ");
            if found_header {
                return Some(txt.to_string())
            }
            else if txt == *header_name {
                // We store the value of the next sibling cell as the name
                found_header = true;
            }
        }
    }
    return None
}

fn insert_water_detail(water_detail: &WaterDetail) -> Result<i64, rusqlite::Error> {
    let conn = rusqlite::Connection::open("./water_buyer_relationships.db3").unwrap();
    let mut stmt = conn.prepare(INSERT_WATER_DETAIL_SQL).unwrap();
    let result = stmt.insert(rusqlite::named_params! {
        ":water_system_no": water_detail.ws_number,
        ":water_system_name": water_detail.name,
        ":state_code": water_detail.st_code,
        ":is_no": water_detail.is_number,
    });
    return result
}

fn insert_buyer_seller_relationship(relationship: &BuyerSellerRelationship) -> Result<i64, rusqlite::Error> {
    let conn = rusqlite::Connection::open("./water_buyer_relationships.db3").unwrap();
    let mut stmt = conn.prepare(INSERT_BUYER_SELLER_RELATIONSHIP_SQL).unwrap();
    let result = stmt.insert(rusqlite::named_params! {
        ":seller": relationship.seller,
        ":buyer": relationship.buyer,
        ":population": relationship.population,
        ":availability": relationship.availability
    });
    return result
}
