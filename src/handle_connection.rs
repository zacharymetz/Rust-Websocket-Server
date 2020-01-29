
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::fs;
use std::str;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use base64::{encode, decode};
/*
    This is the function that will handle the connection,
    figure out if its an http request or websocket 
*/

fn main(){


}

pub fn route(mut stream: TcpStream) {
    
    //  lets snag the request header and then do something 
    //  with it like figure out 
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    //  now that we have pasred the headers 
    let mut request_header = HTTPHeader::from_str(str::from_utf8(&buffer).unwrap()).unwrap();// parse_header(str::from_utf8(&buffer).unwrap().to_string());
    
    //  figure out if tis a websocket or http 
    if request_header.connection_type == "http" {
        // if its an http request send back http okay headers 
        let contents = fs::read_to_string("hello.html").unwrap();

        let response = format!("HTTP/1.1 200 OK\r\n\r\n{}\n", contents);

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();

    }else if request_header.connection_type == "ws" {
        //  if your a websocket send back websock headers 

        // create a Sha1 object
        let mut hasher = Sha1::new();

        // write input message
        // 
        let preHash =  request_header.header_options.get("Sec-WebSocket-Key").unwrap().clone()+"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
        hasher.input_str(&preHash);

        // read hash digest
        let mut hash_output = [0 ; 20];
        hasher.result(&mut hash_output);
        println!("{}",&hash_output[0]);
        let hex = encode(&hash_output);
        println!("client websocket key : {}", request_header.header_options.get("Sec-WebSocket-Key").unwrap());
        println!("client hashed    key : {}", hex);

        let mut response = String::from( "HTTP/1.1 101 Switching Protocols
Upgrade: websocket\r\n\
Connection: Upgrade\r\n\
Sec-WebSocket-Accept: ")+ &hex +"
\r\n\r\n".clone();

        println!("response {}",response);
        stream.write(response.as_bytes()).unwrap();
    }

    

    
}


/*
    HTTPHeader Code is here so move to a new file in a bit cuz it works 
*/
use std::collections::HashMap;
use std::str::FromStr;

pub struct HTTPHeader{
    method : String,
    url : String,
    httpversion : String,
    connection_type : String,
    header_options : HashMap<String, String>
}


impl FromStr for HTTPHeader {
    type Err = std::num::ParseIntError;

    // Parses a color hex code of the form '#rRgGbB..' into an
    // instance of 'RGB'
    fn from_str(header: &str) -> Result<Self, Self::Err> {
    
        

        let mut header_lines = header.split("\n");
        let line = header_lines.next().unwrap();


        //  get the first line and figure out the method
        //  url and the http version
        let mut first_line = line.split(" ");
        let method = first_line.next().unwrap();
        let url = first_line.next().unwrap();
        let httpversion = first_line.next().unwrap();
        
        //  for each line lets do something 
        let mut header_options = HashMap::new();

        for header_line in header_lines{
            //  when we get to this loop what happens is we split at the :
            //  then we get the options and the thing 

            
            let mut line = header_line.split(": ").clone();

            let mut lineCount = line.clone().count();
            if lineCount >1 {   //  other lines are blank or \n | \r line lines 
                
                header_options.insert(remove_whitespace( line.next().unwrap()),
                                remove_whitespace(line.next().unwrap()));
            }

            
            
            
        }

        // println!("Method  : {}",method);
        // println!("Url     : {}",url);
        // println!("version : {}",httpversion);
        // let size = header_options.keys().len();
        // for key in header_options.keys(){
        // println!("key is  : {}", key);
        // println!("part is : {}", header_options.get(key).unwrap());
        
        // }
        // println!("options : {}", size);

        //  figure out the request type 
        let upgrade_option = header_options.get("Upgrade");
        
        if upgrade_option.is_some() && upgrade_option.unwrap() == "websocket"{
            
                //  its a websocket type 
                println!("New Websocket Connection");
                Ok(HTTPHeader{
                    method : method.to_string(),
                    url : url.to_string(),
                    httpversion : url.to_string(),
                    connection_type : "ws".to_string(),
                    header_options : header_options
                })
            
            
        }else{
            //  its an http request 
            println!("New Http Connection");
            Ok(HTTPHeader{
                method : method.to_string(),
                url : url.to_string(),
                httpversion : url.to_string(),
                connection_type : "http".to_string(),
                header_options : header_options
            })
        }


    //  create a new structure here and pass the thing through 
    
    }
}
//  as the name says removes any whitespace on this thing
fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}