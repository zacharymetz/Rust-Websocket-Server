
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::fs;
use std::str;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use base64::{encode};
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
        //println!("{}",&hash_output[0]);
        let hex = encode(&hash_output);
        println!("client websocket key : {}", request_header.header_options.get("Sec-WebSocket-Key").unwrap());
        println!("client hashed    key : {}", hex);

        let mut response = String::from( "HTTP/1.1 101 Switching Protocols
Upgrade: websocket\r\n\
Connection: Upgrade\r\n\
Sec-WebSocket-Accept: ")+ &hex +"
\r\n\r\n".clone();

        
        stream.write(response.as_bytes()).unwrap();
        //  we are here lets make a new object for 
        //  the websocket connection data 
        let mut websocket = new_websocket(stream);

        //  now we wait for a message from them and just the header
        let mut header_buffer = [0; 14];
        //  
        websocket.stream.read(&mut header_buffer).unwrap();
        //  here we can do sometrhing with they bytes 
        //  so lets parse the packet ??? 
        parse_packet(&header_buffer,websocket);
        //println!("{:?}",buffer.bytes());
        while true {
            parse_packet(&header_buffer,websocket);
        }
        

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


//  data frame stuff here 
pub struct WebSocketConnection{
    stream : TcpStream,
    disconnect : bool

}

fn parse_packet(& buffer : &[u8; 14],mut websocket: WebSocketConnection){
    //  first get the fin packet number  (the last bit of the first one )
    let fin = get_bit_at(buffer[0],7); 
    //  then get the mask   (last bit of the seond one )
    let mask = get_bit_at(buffer[1],7);
    //  then get the op code  (first 4 of the first one)
    let opcode = (buffer[0] & 0b0000_1111) as u8;
    //  payload length (the first 7 of the 2nd one)
    let mut payload_length = (buffer[0] & 0b0111_1111) as u64;
    //  extended payload length 
    if payload_length == 126 {
        //  interprest the next 16 bits as a number cuz that is the payload length
        payload_length = ((buffer[2] as u64) << 8) | buffer[3] as u64;
    } else if payload_length == 127 {
        //  interprest the next 64 bits as the payload length 
        payload_length = ((buffer[2] as u64) << 56) | ((buffer[3] as u64) << 48) |
                         ((buffer[4] as u64) << 40) | ((buffer[5] as u64) << 32) |
                         ((buffer[6] as u64) << 24) | ((buffer[7] as u64) << 16) | 
                         ((buffer[8] as u64) << 8) | buffer[9] as u64;

    }


    //  payload data
        //  alocate a buffer with all length of the data from the 
        //  frame 
    let payload_data_length = payload_length as usize;
        //  we must see if it is 32 bits 
    let mut payload_data = vec![0;payload_data_length];
    //  copy the remaining bits from the buffer thing
    websocket.stream.read(&mut payload_data).unwrap();
    //  use the stream to get the rest of the payload bytes 
    println!("payload Bytes : {:?}",payload_data.bytes())
    

}

pub fn new_websocket(stream: TcpStream) -> WebSocketConnection{
    WebSocketConnection{
        stream: stream.try_clone().expect("clone failed..."),
        disconnect : false
    }
}

/// gets the bit at position `n`. Bits are numbered from 0 (least significant) to 31 (most significant).
fn get_bit_at(input: u8, n: u8) -> bool {
    if n < 32 {
        input & (1 << n) != 0
    } else {
        false
    }
}